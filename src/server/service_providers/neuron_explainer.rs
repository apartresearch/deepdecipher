use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::data_types::NeuronExplainer as NeuronExplainerData;
use crate::data::retrieve::neuron_explainer;
use crate::data::{DataObjectHandle, Database, ModelHandle, NeuronIndex};
use crate::server::State;

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuronExplainer;

async fn data_object(state: &State, model: &ModelHandle) -> Result<NeuronExplainerData> {
    let database = state.database();
    let data_object_name = "neuron_explainer";
    let data_object = database
        .data_object(data_object_name)
        .await?
        .with_context(|| format!("No data object with name '{data_object_name}'."))?;
    model.data_object(&data_object).await.with_context(|| {
        format!(
            "Failed to get neuron explainer data object for model '{}'.",
            model.name()
        )
    })
}

#[async_trait]
impl ServiceProviderTrait for NeuronExplainer {
    async fn required_data_objects(&self, database: &Database) -> Result<Vec<DataObjectHandle>> {
        let data_object_name = "neuron_explainer";
        let data_object = database
            .data_object(data_object_name)
            .await?
            .with_context(|| format!("No data object with name '{data_object_name}'. This should have been checked when the service was created."))?;
        Ok(vec![data_object])
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
        let index = NeuronIndex{layer: layer_index, neuron: neuron_index};
        let page = if let Some(page) = data_object(state, model)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await? {
                page
            } else {
                neuron_explainer::fetch_neuron(&Client::new(), neuron_explainer::model_url(model.name(), index)?).await.with_context(|| 
                    format!("No neuron explainer page exists for neuron {index} in model '{model_name}' and fetching from source failed.", 
                        model_name = model.name()
                    )
                )?
            };
        Ok(json!(page))
    }
}
