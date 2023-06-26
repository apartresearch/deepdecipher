use std::{fs, path::Path};

use anyhow::{Context, Result};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{data::NeuronIndex, server::State};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2Graph;

#[async_trait]
impl ServiceProviderTrait for Neuron2Graph {
    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let path = Path::new("data")
            .join(model)
            .join("neuron2graph")
            .join(format!("layer_{layer_index}",))
            .join(format!("{layer_index}_{neuron_index}"))
            .join("graph");
        let graph = fs::read_to_string(path).map(|page| json!(page)).with_context(|| format!("Failed to read neuron2graph page for neuron {neuron_index} in layer {layer_index} of model '{model}'."))?;
        let similar_neurons = state
            .neuron_store(model)
            .await?
            .similar_neurons(
                NeuronIndex {
                    layer: layer_index,
                    neuron: neuron_index,
                },
                0.4,
            )?
            .into_iter()
            .map(
                |(
                    NeuronIndex {
                        layer: layer_index,
                        neuron: neuron_index,
                    },
                    similarity,
                )| {
                    json!({
                        "layer": layer_index,
                        "neuron": neuron_index,
                        "similarity": similarity,
                    })
                },
            )
            .collect::<Vec<_>>();
        Ok(json!({
        "graph": graph,
        "similar": similar_neurons,}))
    }
}
