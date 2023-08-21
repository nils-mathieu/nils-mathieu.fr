use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// The configuration of the server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The address to bind to.
    pub address: SocketAddr,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            address: ([0, 0, 0, 0], 8080).into(),
        }
    }
}
