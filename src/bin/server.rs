use anyhow::Result;
use clap::Parser;
use deepdecipher::cli::ServerConfig;

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::init();

    ServerConfig::parse().start().await
}
