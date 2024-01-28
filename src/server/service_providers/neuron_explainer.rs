use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::service_provider::{NoData, ServiceProviderTrait};
use crate::{
    data::{
        data_objects::NeuronExplainerPage, data_types::NeuronExplainer as NeuronExplainerData,
        retrieve::neuron_explainer, DataTypeHandle, Database, ModelHandle, NeuronIndex,
    },
    server::State,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuronExplainer;

async fn data_type(state: &State, model: &ModelHandle) -> Result<NeuronExplainerData> {
    let database = state.database();
    let data_type_name = "neuron_explainer";
    let data_type = database
        .data_type(data_type_name)
        .await?
        .with_context(|| format!("No data object with name '{data_type_name}'."))?;
    model.data_type(&data_type).await.with_context(|| {
        format!(
            "Failed to get neuron explainer data object for model '{}'.",
            model.name()
        )
    })
}

#[async_trait]
impl ServiceProviderTrait for NeuronExplainer {
    type ModelPageObject = NoData;
    type LayerPageObject = NoData;
    type NeuronPageObject = NeuronExplainerPage;

    async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>> {
        let data_type_name = "neuron_explainer";
        let data_type = database.data_type(data_type_name).await?.with_context(|| {
            format!(
                "No data object with name '{data_type_name}'. This should have been checked when \
                 the service was created."
            )
        })?;
        Ok(vec![data_type])
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
        let index = NeuronIndex {
            layer: layer_index,
            neuron: neuron_index,
        };
        let page = if let Some(page) = data_type(state, model)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await?
        {
            page
        } else {
            neuron_explainer::fetch_neuron(
                &Client::new(),
                neuron_explainer::model_url(model.name(), index)?,
            )
            .await
            .with_context(|| {
                format!(
                    "No neuron explainer page exists for neuron {index} in model '{model_name}' \
                     and fetching from source failed.",
                    model_name = model.name()
                )
            })?
        };
        Ok(page)
    }
}
