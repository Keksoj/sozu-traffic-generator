use std::sync::Arc;

use once_cell::sync::Lazy;
use prometheus::{register_int_counter_vec, IntCounterVec};
use sozu_client::{
    channel::ConnectionProperties, config::canonicalize_command_socket, Client, Sender,
};
use sozu_command_lib::{
    config::Config,
    proto::{command::Request, display::format_request_type},
};
use tokio::task::JoinError;
use tracing::{debug, error, info, trace, warn};

use crate::svc::config::GeneratorConfiguration;

use super::requests;

// -----------------------------------------------------------------------------
// Telemetry

static REQUEST_EMITTED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "traffic_generator_request_emitted",
        "Number of request emitted by the traffic generator",
        &["kind"]
    )
    .expect("'traffic-generator' to not be already registered")
});

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Could not create a listener")]
    ListenerCreation(String),
    #[error("The request to send is empty")]
    EmptyRequest,
    #[error("failed to create Sōzu client, {0}")]
    CreateClient(sozu_client::Error),
    #[error("failed to canonicalize path to command socket, {0}")]
    CanonicalizeSocket(sozu_client::config::Error),
    #[error("failed to retrieve hostname to use as consumer name, {0}")]
    Hostname(std::io::Error),
    #[error("failed to send batch message to Sōzu, {0}")]
    Send(sozu_client::Error),
    #[error("failed to spawn task on tokio runtime, {0}")]
    TokioSpawn(JoinError),
    #[error("failed to create temporary directory, {0}")]
    CreateTempDir(std::io::Error),
    #[error("failed to create temporary file, {0}")]
    CreateFile(std::io::Error),
    #[error("failed to serialize request, {0}")]
    Serde(serde_json::Error),
    #[error("failed to write request on writer, {0}")]
    Write(std::io::Error),
    #[error("failed to flush request of writer on disk, {0}")]
    Flush(std::io::Error),
}

impl From<JoinError> for GeneratorError {
    #[tracing::instrument]
    fn from(err: JoinError) -> Self {
        Self::TokioSpawn(err)
    }
}

pub struct Generator {
    /// Generator configuration
    config: Arc<GeneratorConfiguration>,
    /// Pooled client to Sōzu
    client: Client,
}

impl Generator {
    #[tracing::instrument(skip_all)]
    pub async fn try_new(
        config: Arc<GeneratorConfiguration>,
        sozu_config: Arc<Config>,
    ) -> Result<Self, GeneratorError> {
        info!("Create Sōzu client");
        let mut opts = ConnectionProperties::from(&*sozu_config);
        if opts.socket.is_relative() {
            opts.socket = canonicalize_command_socket(&config.sozu.configuration, &sozu_config)
                .map_err(GeneratorError::CanonicalizeSocket)?;
        }

        let client = Client::try_new(opts)
            .await
            .map_err(GeneratorError::CreateClient)?;

        Ok(Self { config, client })
    }

    ///
    #[tracing::instrument(skip_all)]
    pub async fn add_a_random_cluster_on_sozu(&mut self) -> Result<(), GeneratorError> {
        debug!("Generating a bunch of requests for a random cluster");
        let requests = requests::generate_requests_for_a_random_cluster()?;

        debug!("Sending the requests");
        self.send_requests_to_sozu(requests).await
    }

    #[tracing::instrument(skip_all)]
    pub async fn send_requests_to_sozu(
        &mut self,
        requests: Vec<Request>,
    ) -> Result<(), GeneratorError> {
        for request in requests {
            let request_type = request.request_type.ok_or(GeneratorError::EmptyRequest)?;

            REQUEST_EMITTED
                .with_label_values(&[&format_request_type(&request_type)])
                .inc();

            trace!("Sending request {:?}", request_type);
            self.client
                .send(request_type)
                .await
                .map_err(GeneratorError::Send)?;

            std::thread::sleep(std::time::Duration::from_millis(
                self.config.sleep_between_requests,
            ));
        }

        Ok(())
    }
}
