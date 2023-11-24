//! # Messaging module
//!
//! This module provides the logic to generate requests and send them to Sōzu

use std::{sync::Arc, time::Duration};

use sozu_command_lib::config::Config;
use tracing::info;

use self::generator::{Generator, GeneratorError};

use super::config::GeneratorConfiguration;

pub mod generator;
pub mod requests;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to create Generator, {0}")]
    CreateGenerator(GeneratorError),
    #[error("failed to connect and consume topic, {0}")]
    Connect(GeneratorError),
}

/// send a bunch of messages to Sōzu every 5 seconds
#[tracing::instrument(skip_all)]
pub async fn generate(
    config: Arc<GeneratorConfiguration>,
    sozu_config: Arc<Config>,
) -> Result<(), Error> {
    let mut generator = Generator::try_new(config.to_owned(), sozu_config)
        .await
        .map_err(Error::CreateGenerator)?;

    info!("Ready to generate messages");

    for i in 0..config.clusters_to_send {
        generator
            .add_a_random_cluster_on_sozu()
            .await
            .map_err(Error::Connect)?;
        info!("Sent {} cluster out of {}", i, config.clusters_to_send);
    }
    Ok(())
}
