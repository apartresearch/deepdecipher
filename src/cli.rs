use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use crate::server;

#[derive(Parser, Clone, Debug)]
pub struct ServerConfig {
    database_path: PathBuf,
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, short = 'l')]
    log_path: Option<PathBuf>,
}

impl ServerConfig {
    pub async fn start(self) -> Result<()> {
        server::start_server(self).await
    }

    pub fn database_path(&self) -> &Path {
        self.database_path.as_path()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn log_path(&self) -> Option<&Path> {
        self.log_path.as_deref()
    }
}
