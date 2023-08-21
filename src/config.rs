use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// The configuration of the server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The address to bind to.
    pub address: SocketAddr,
    /// The path to the public SSL certificate.
    pub ssl_cert: PathBuf,
    /// The path to the private SSL key.
    pub ssl_key: PathBuf,
}

impl Config {
    /// Loads a [`Config`] instance from the default configuration file.
    ///
    /// # Panics
    ///
    /// This function panics if the configuration file cannot be read.
    pub fn try_load() -> ron::error::SpannedResult<Self> {
        let file = std::fs::File::open("config.ron")?;
        ron::de::from_reader(file)
    }
}
