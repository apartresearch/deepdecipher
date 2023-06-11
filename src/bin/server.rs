use anyhow::{Context, Result};
use neuronav::server;

pub fn main() -> Result<()> {
    server::start_server().context("Failed to start server.")
}
