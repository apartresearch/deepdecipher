use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::{self, NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage},
    server::State,
};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuroscope;

#[async_trait]
impl ServiceProviderTrait for Neuroscope {
    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let database = state.database().await?;
        let model = database
            .model(model_name.to_owned())
            .await?
            .with_context(|| format!("No model with name {model_name}."))?;
        let data_object = model
            .get_data_object(&database, "neuroscope")
            .await
            .with_context(|| {
                format!("Failed to get neuroscope data object for model '{model_name}'.")
            })?;
        let page = data_object
            .neuroscope()
            .context("Type of 'neuroscope' data object must be 'neuroscope'.")?
            .model_page(&database, &model)
            .await?;
        Ok(json!(page))
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
        let model = database
            .model(model_name.to_owned())
            .await?
            .with_context(|| format!("No model with name {model_name}."))?;
        let data_object = model.get_data_object(&database, "neuroscope").await?;
        let page = data_object
            .neuroscope()
            .context("Type of 'neuroscope' data object must be 'neuroscope'.")?
            .layer_page(&database, &model, layer_index)
            .await?;
        Ok(json!(page))
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let database = state.database().await?;
        let model = database
            .model(model_name.to_owned())
            .await?
            .with_context(|| format!("No model with name {model_name}."))?;
        let data_object = model.get_data_object(&database, "neuroscope").await?;
        let page = data_object
            .neuroscope()
            .context("Type of 'neuroscope' data object must be 'neuroscope'.")?
            .neuron_page(&database, &model, layer_index, neuron_index)
            .await?;
        Ok(json!(page))
    }
}
