//! # Sozu traffic generator
//!
//! This application sends a lot of routing instructions to Sōzu

use std::{path::PathBuf, sync::Arc};

use clap::{ArgAction, Parser};
use tracing::{error, info};

use crate::svc::{
    config::{self, GeneratorConfiguration},
    logging::{self, LoggingInitGuard},
    messaging,
};

pub mod svc;

// -----------------------------------------------------------------------------
// Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to send requests to Sōzu")]
    Messaging(messaging::Error),
    #[error("failed to load configuration, {0}")]
    Configuration(config::Error),
    #[error("failed to initialize the logging system, {0}")]
    Logging(logging::Error),
    #[error("failed to create handler on termination signal, {0}")]
    Termination(std::io::Error),
    #[error("failed to load sōzu configuration, {0}")]
    SozuConfiguration(sozu_client::config::Error),
}

// -----------------------------------------------------------------------------
// Args

/// A traffic generator that sends requests to Sōzu to add random clusters to its state
#[derive(Parser, PartialEq, Eq, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Increase verbosity
    #[clap(short = 'v', global = true, action = ArgAction::Count)]
    pub verbosity: u8,
    /// Path to the configuration file of the traffic generator
    #[clap(short = 'c', long = "config")]
    pub config: Option<PathBuf>,
}

impl paw::ParseArgs for Args {
    type Error = Error;

    fn parse_args() -> Result<Self, Self::Error> {
        Ok(Self::parse())
    }
}

// -----------------------------------------------------------------------------
// main

#[paw::main]
#[tokio::main(flavor = "current_thread")]
pub async fn main(args: Args) -> Result<(), Error> {
    // -------------------------------------------------------------------------
    // Retrieve configuration
    let config = Arc::new(match &args.config {
        Some(path) => {
            GeneratorConfiguration::try_from(path.to_owned()).map_err(Error::Configuration)?
        }
        None => GeneratorConfiguration::try_new().map_err(Error::Configuration)?,
    });

    // -------------------------------------------------------------------------
    // Initialize logging system
    let _guard = logging::initialize(args.verbosity as usize)
        .map(|_| LoggingInitGuard::default())
        .map_err(Error::Logging)?;
    info!("initialized the logger");

    let sozu_config = Arc::new(
        sozu_client::config::try_from(&config.sozu.configuration)
            .map_err(Error::SozuConfiguration)?,
    );

    let result = tokio::select! {
        r = tokio::signal::ctrl_c() => r.map_err(Error::Termination),
        r = svc::messaging::generate(config,sozu_config) => r.map_err(Error::Messaging),
    };

    if let Err(err) = result {
        error!(
            error = err.to_string(),
            "Could not execute {} properly",
            env!("CARGO_PKG_NAME")
        );

        return Err(err);
    }

    info!("Gracefully halted {}!", env!("CARGO_PKG_NAME"));
    Ok(())
}
