use std::{
    fs::{self, File},
    path::Path,
};

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub num_neurons: u32,
}
