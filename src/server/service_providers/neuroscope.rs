use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::data_types::Neuroscope as NeuroscopeData;
use crate::data::ModelHandle;
use crate::server::State;

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuroscope;

async fn data_object(state: &State, model: &ModelHandle) -> Result<NeuroscopeData> {
    let database = state.database();
    let data_object_name = "neuroscope";
    let data_object = database
        .data_object(data_object_name)
        .await?
        .with_context(|| format!("No data object with name '{data_object_name}'."))?;
    model.data_object(&data_object).await.with_context(|| {
        format!(
            "Failed to get neuroscope data object for model '{}'.",
            model.name()
        )
    })
}

#[async_trait]
impl ServiceProviderTrait for Neuroscope {
    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let page = data_object(state, model).await?.model_page().await?;
        Ok(json!(page))
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let page = data_object(state, model)
            .await?
            .layer_page(layer_index)
            .await?;
        Ok(json!(page))
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let page = data_object(state, model)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await?;
        Ok(json!(page))
    }
}
