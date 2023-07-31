use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    data::{DataObjectHandle, Database, ModelHandle},
    server::State,
};

use super::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata;

#[async_trait]
impl ServiceProviderTrait for Metadata {
    async fn required_data_objects(&self, _database: &Database) -> Result<Vec<DataObjectHandle>> {
        Ok(vec![])
    }

    async fn model_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let metadata = serde_json::to_value(model.metadata())?;
        Ok(metadata)
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        _layer_index: u32,
    ) -> Result<serde_json::Value> {
        let metadata = serde_json::to_value(model.metadata())?;
        Ok(metadata)
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        _layer_index: u32,
        _neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let metadata = serde_json::to_value(model.metadata())?;
        Ok(metadata)
    }
}
