use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use crate::server;

#[derive(Parser, Clone, Debug)]
pub struct ServerConfig {
    database_path: PathBuf,
    #[arg(long, default_value = "localhost")]
    url: String,
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, short = 'l')]
    log_path: Option<PathBuf>,
    #[arg(short = 'w')]
    num_workers: Option<usize>,
}

impl ServerConfig {
    pub async fn start(self) -> Result<()> {
        server::start_server(self).await
    }

    pub fn database_path(&self) -> &Path {
        self.database_path.as_path()
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn log_path(&self) -> Option<&Path> {
        self.log_path.as_deref()
    }

    pub fn num_workers(&self) -> Option<usize> {
        self.num_workers
    }
}
