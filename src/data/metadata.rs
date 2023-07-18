use std::{
    fs::{self, File},
    path::Path,
};

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

use super::NeuronIndex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub layers: Vec<LayerMetadata>,
    pub activation_function: String,
    pub num_total_neurons: u32,
    pub num_total_parameters: u32,
    pub dataset: String,
}

impl Metadata {
    pub fn to_file<P: AsRef<Path>>(&self, data_path: P) -> Result<()> {
        let model_metadata_path = data_path.as_ref().join(&self.name).join("metadata.json");
        fs::create_dir_all(
            model_metadata_path
                .parent()
                .with_context(|| format!("Invalid path '{model_metadata_path:?}'"))?,
        )
        .with_context(|| format!("Failed to create directory for '{model_metadata_path:?}'"))?;
        let model_metadata_file = File::create(model_metadata_path)?;

        serde_json::to_writer(model_metadata_file, self)?;
        Ok(())
    }

    pub fn num_layers(&self) -> u32 {
        self.layers.len() as u32
    }

    pub fn neuron_indices(&self) -> impl Iterator<Item = NeuronIndex> {
        self.layers
            .iter()
            .map(|layer_metadata| layer_metadata.num_neurons)
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .flat_map(|(layer_index, num_neurons)| {
                (0..num_neurons).map(move |neuron_index| NeuronIndex {
                    layer: layer_index as u32,
                    neuron: neuron_index,
                })
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub num_neurons: u32,
}
