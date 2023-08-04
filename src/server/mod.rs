use anyhow::Result;

use crate::data::{database::Database, ModelHandle};

mod service;
pub use service::Service;
mod service_providers;
pub use service_providers::ServiceProvider;
mod start;
pub use start::start_server;
mod response;

pub struct State {
    database: Database,
}

impl State {
    pub fn database(&self) -> &Database {
        &self.database
    }
}

pub async fn metadata_value(model_handle: &ModelHandle) -> Result<serde_json::Value> {
    let mut metadata = serde_json::to_value(model_handle.metadata())?;
    let available_services: Vec<_> = model_handle
        .available_services()
        .await?
        .into_iter()
        .map(|service| service.name().to_owned())
        .collect();
    metadata["available_services"] = serde_json::to_value(available_services)?;
    Ok(metadata)
}
