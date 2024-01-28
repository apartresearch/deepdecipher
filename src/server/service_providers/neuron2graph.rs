use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::service_provider::{NoData, ServiceProviderTrait};
use crate::{
    data::{
        data_objects::Neuron2GraphData as Neuron2GraphDataObject,
        data_types::{Neuron2Graph as Neuron2GraphData, NeuronStore as NeuronStoreData},
        ModelHandle,
    },
    server::State,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2Graph;

async fn data_type(
    state: &State,
    model: &ModelHandle,
) -> Result<(Neuron2GraphData, NeuronStoreData)> {
    let model_name = model.name();
    let database = state.database();
    let n2g_object_name = "neuron2graph";
    let n2g_data_type = database
        .data_type(n2g_object_name)
        .await?
        .with_context(|| format!("No data object with name '{n2g_object_name}'."))?;
    let n2g_data_type = model.data_type(&n2g_data_type).await.with_context(|| {
        format!("Failed to get neuron2graph data object for model '{model_name}'.")
    })?;

    let neuron_store_object_name = "neuron_store";
    let neuron_store_data_type: crate::data::DataTypeHandle = database
        .data_type(neuron_store_object_name)
        .await?
        .with_context(|| format!("No data object with name '{neuron_store_object_name}'."))?;
    let neuron_store_data_type = model
        .data_type(&neuron_store_data_type)
        .await
        .with_context(|| {
            format!("Failed to get neuron store data object for model '{model_name}'.")
        })?;

    Ok((n2g_data_type, neuron_store_data_type))
}

#[async_trait]
impl ServiceProviderTrait for Neuron2Graph {
    type ModelPageObject = NoData;
    type LayerPageObject = NoData;
    type NeuronPageObject = Neuron2GraphDataObject;

    async fn required_data_types(
        &self,
        database: &crate::data::Database,
    ) -> Result<Vec<crate::data::DataTypeHandle>> {
        let n2g_object_name = "neuron2graph";
        let neuron_store_object_name = "neuron_store";
        let n2g_data_type = database
            .data_type(n2g_object_name)
            .await?
            .with_context(|| {
                format!(
                    "No data object with name '{n2g_object_name}'. This should have been checked \
                     when service was created."
                )
            })?;
        let neuron_store_data_type = database
            .data_type(neuron_store_object_name)
            .await?
            .with_context(|| {
                format!(
                    "No data object with name '{neuron_store_object_name}'. This should have been \
                     checked when service was created."
                )
            })?;
        Ok(vec![n2g_data_type, neuron_store_data_type])
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
        let (n2g_data_type, neuron_store_data_type) = data_type(state, model).await?;
        let graph = n2g_data_type
            .neuron_graph(layer_index, neuron_index)
            .await?;
        let similar = neuron_store_data_type
            .neuron_similarities(layer_index, neuron_index)
            .await?;
        Ok(Neuron2GraphDataObject::new(graph, similar))
    }
}
