use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::service_provider::ServiceProviderTrait;
use crate::{
    data::{
        data_objects::{NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage},
        data_types::Neuroscope as NeuroscopeData,
        retrieve::neuroscope::scrape_neuron_page,
        DataTypeHandle, Database, ModelHandle, NeuronIndex,
    },
    server::State,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuroscope;

async fn data_type(state: &State, model: &ModelHandle) -> Result<NeuroscopeData> {
    let database = state.database();
    let data_type_name = "neuroscope";
    let data_type = database
        .data_type(data_type_name)
        .await?
        .with_context(|| format!("No data object with name '{data_type_name}'."))?;
    model.data_type(&data_type).await.with_context(|| {
        format!(
            "Failed to get neuroscope data object for model '{}'.",
            model.name()
        )
    })
}

#[async_trait]
impl ServiceProviderTrait for Neuroscope {
    type ModelPageObject = NeuroscopeModelPage;
    type LayerPageObject = NeuroscopeLayerPage;
    type NeuronPageObject = NeuroscopeNeuronPage;

    async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>> {
        let data_type_name = "neuroscope";
        let data_type = database.data_type(data_type_name).await?.with_context(|| {
            format!(
                "No data object with name '{data_type_name}'. This should have been checked when \
                 the service was created."
            )
        })?;
        Ok(vec![data_type])
    }

    async fn model_object(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<Self::ModelPageObject> {
        data_type(state, model)
            .await?
            .model_page()
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuroscope model page for model '{}'.",
                    model.name()
                )
            })
    }

    async fn layer_object(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<Self::LayerPageObject> {
        data_type(state, model)
            .await?
            .layer_page(layer_index)
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuroscope layer page for layer {} in model '{}'.",
                    layer_index,
                    model.name()
                )
            })
    }

    async fn neuron_object(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Self::NeuronPageObject> {
        data_type(state, model)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuroscope neuron page for neuron {neuron_index} in model \
                     '{model_name}'.",
                    neuron_index = NeuronIndex {
                        layer: layer_index,
                        neuron: neuron_index
                    },
                    model_name = model.name()
                )
            })?
            .with_context(|| {
                format!(
                    "Failed to get neuroscope neuron page for neuron {neuron_index} in model \
                     '{model_name}'.",
                    neuron_index = NeuronIndex {
                        layer: layer_index,
                        neuron: neuron_index
                    },
                    model_name = model.name()
                )
            })
    }

    async fn layer_json(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let page = data_type(state, model)
            .await?
            .layer_page(layer_index)
            .await?;
        Ok(json!(page))
    }

    async fn neuron_json(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let page = if let Some(page) = data_type(state, model)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await?
        {
            page
        } else {
            scrape_neuron_page(
                model.name(),
                NeuronIndex {
                    layer: layer_index,
                    neuron: neuron_index,
                },
            )
            .await
            .with_context(|| {
                format!(
                    "No neuroscope page exists for neuron l{layer_index}n{neuron_index} in model \
                     '{model_name}' and fetching from source failed.",
                    model_name = model.name()
                )
            })?
        };
        Ok(json!(page))
    }
}
