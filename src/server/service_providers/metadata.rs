use std::fs;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{data::Metadata as MetadataData, server::State};

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
        let database = state.database().await?;
        let model = database.model(model_name.to_owned()).await?;
        let metadata = serde_json::to_value(model.metadata())?;
        Ok(metadata)
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let database = state.database().await?;
        let model = database.model(model_name.to_owned()).await?;
        let metadata = serde_json::to_value(
            model
                .metadata()
                .layers
                .get(layer_index as usize)
                .with_context(|| {
                    format!(
                        "Layer index {layer_index} out of bounds. Model only has {} layers.",
                        model.metadata().layers.len()
                    )
                })?,
        )?;
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
