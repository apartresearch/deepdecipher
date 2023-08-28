use anyhow::Result;
use clap::Parser;
use deepdecipher::cli::ServerConfig;

#[tokio::main]
pub async fn main() -> Result<()> {
    ServerConfig::parse().start().await
}
