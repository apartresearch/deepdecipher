use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use crate::server;

#[derive(Parser, Clone, Debug)]
pub struct ServerConfig {
    database_path: PathBuf,
}

impl ServerConfig {
    pub async fn start(self) -> Result<()> {
        server::start_server(self).await
    }

    pub fn database_path(&self) -> &Path {
        self.database_path.as_path()
    }
}
