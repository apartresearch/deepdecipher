use anyhow::{Context, Result};
use neuronav::server;

pub fn main() -> Result<()> {
    env_logger::init();

    server::start_server().context("Failed to start server.")
}
