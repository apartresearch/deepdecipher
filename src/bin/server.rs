use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use deepdecipher::{data::Database, server};
use tokio::runtime::Runtime;

pub fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let database_path = PathBuf::from(
        args.get(1)
            .context("Please provide the database path as the first argument.")?,
    );
    let database_path = database_path.as_path();

    let database = Runtime::new()
        .context("Failed to start async runtime to open database.")?
        .block_on(async {
            if database_path.exists() {
                Database::open(database_path).await.with_context(|| {
                    format!("Failed to open database at path '{database_path:?}'.")
                })
            } else {
                Database::initialize(database_path).await.with_context(|| {
                    format!("Failed to initialize database at path '{database_path:?}'.")
                })
            }
        })?;

    server::start_server(database).context("Failed to start server.")
}
