//! # Configuration module
//!
//! This module provides structures and helpers to interact with the configuration

use std::{
    env::{self, VarError},
    net::SocketAddr,
    path::PathBuf,
};

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to build configuration, {0}")]
    Build(ConfigError),
    #[error("failed to serialize configuration, {0}")]
    Serialize(ConfigError),
    #[error("failed to retrieve environment variable '{0}', {1}")]
    EnvironmentVariable(&'static str, VarError),
}

// -----------------------------------------------------------------------------
// Sōzu

/// Sōzu-related configuration
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Sozu {
    /// Path to configuration file
    #[serde(rename = "configuration")]
    pub configuration: PathBuf,
    /// Listener socket address
    #[serde(rename = "listener")]
    pub listener: SocketAddr,
}

// -----------------------------------------------------------------------------
// Configuration

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct GeneratorConfiguration {
    /// Socket address on which expose metrics server
    #[serde(rename = "listening-address")]
    pub listening_address: SocketAddr,
    /// Sōzu configuration
    #[serde(rename = "sozu")]
    pub sozu: Sozu,
}

impl TryFrom<PathBuf> for GeneratorConfiguration {
    type Error = Error;

    #[tracing::instrument]
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Config::builder()
            .add_source(File::from(path).required(true))
            .build()
            .map_err(Error::Build)?
            .try_deserialize()
            .map_err(Error::Serialize)
    }
}

impl GeneratorConfiguration {
    #[tracing::instrument]
    pub fn try_new() -> Result<Self, Error> {
        let homedir = env::var("HOME").map_err(|err| Error::EnvironmentVariable("HOME", err))?;

        Config::builder()
            .add_source(
                File::from(PathBuf::from(format!(
                    "/usr/share/{}/config",
                    env!("CARGO_PKG_NAME")
                )))
                .required(false),
            )
            .add_source(
                File::from(PathBuf::from(format!(
                    "/etc/{}/config",
                    env!("CARGO_PKG_NAME")
                )))
                .required(false),
            )
            .add_source(
                File::from(PathBuf::from(format!(
                    "{}/.config/{}/config",
                    homedir,
                    env!("CARGO_PKG_NAME")
                )))
                .required(false),
            )
            .add_source(
                File::from(PathBuf::from(format!(
                    "{}/.local/share/{}/config",
                    homedir,
                    env!("CARGO_PKG_NAME")
                )))
                .required(false),
            )
            .add_source(File::from(PathBuf::from("config")).required(false))
            .build()
            .map_err(Error::Build)?
            .try_deserialize()
            .map_err(Error::Serialize)
    }
}
