use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::server::State;

use super::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata;

#[async_trait]
impl ServiceProviderTrait for Metadata {
    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let database = state.database();
        let model = database
            .model(model_name.to_owned())
            .await?
            .with_context(|| format!("No model with name {model_name}."))?;
        let metadata = serde_json::to_value(model.metadata())?;
        Ok(metadata)
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
        _layer_index: u32,
    ) -> Result<serde_json::Value> {
        let database = state.database();
        let model = database
            .model(model_name.to_owned())
            .await?
            .with_context(|| format!("No model with name {model_name}."))?;
        let metadata = json!({"layer_size": model.metadata().layer_size});
        Ok(metadata)
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        _model_name: &str,
        _layer_index: u32,
        _neuron_index: u32,
    ) -> Result<serde_json::Value> {
        Ok(json!({}))
    }
}
