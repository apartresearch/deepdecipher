use anyhow::{Context, Result};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::data_types::{Neuron2Graph as Neuron2GraphData, NeuronStore as NeuronStoreData},
    server::State,
};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2Graph;

async fn data_object(
    state: &State,
    model_name: &str,
) -> Result<(Neuron2GraphData, NeuronStoreData)> {
    let database = state.database();
    let model = database
        .model(model_name)
        .await?
        .with_context(|| format!("No model with name {model_name}."))?;
    let n2g_object_name = "neuron2graph";
    let n2g_data_object = database
        .data_object(n2g_object_name)
        .await?
        .with_context(|| format!("No data object with name '{n2g_object_name}'."))?;
    let n2g_data_object = model.data_object(&n2g_data_object).await.with_context(|| {
        format!("Failed to get neuron2graph data object for model '{model_name}'.")
    })?;

    let neuron_store_object_name = "neuron_store";
    let neuron_store_data_object: crate::data::DataObjectHandle = database
        .data_object(neuron_store_object_name)
        .await?
        .with_context(|| format!("No data object with name '{neuron_store_object_name}'."))?;
    let neuron_store_data_object = model
        .data_object(&neuron_store_data_object)
        .await
        .with_context(|| {
            format!("Failed to get neuron store data object for model '{model_name}'.")
        })?;

    Ok((n2g_data_object, neuron_store_data_object))
}

#[async_trait]
impl ServiceProviderTrait for Neuron2Graph {
    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let (n2g_data_object, neuron_store_data_object) = data_object(state, model_name).await?;
        let neuron_graph = n2g_data_object
            .neuron_graph(layer_index, neuron_index)
            .await?;

        let similar_neurons = neuron_store_data_object
            .neuron_similarities(layer_index, neuron_index)
            .await?;

        Ok(json!({
        "graph": neuron_graph,
        "similar": similar_neurons,}))
    }
}
