use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::ModelMetadata;

use super::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata;

impl ServiceProviderTrait for Metadata {
    fn model_page(
        &self,
        _service: &crate::server::Service,
        _state: &crate::server::State,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let path: std::path::PathBuf = Path::new("data").join(model_name).join("metadata.json");
        let text = fs::read_to_string(path)?;
        let metadata = serde_json::from_str(&text)?;
        Ok(metadata)
    }

    fn layer_page(
        &self,
        _service: &crate::server::Service,
        _state: &crate::server::State,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let path = Path::new("data").join(model_name).join("metadata.json");
        let text = fs::read_to_string(path)?;
        let model_metadata: ModelMetadata = serde_json::from_str(&text)?;
        let layer_metadata = &model_metadata
            .layers
            .get(layer_index as usize)
            .context("Layer index out of bounds.")?;
        let metadata = serde_json::to_value(layer_metadata)?;
        Ok(metadata)
    }

    fn neuron_page(
        &self,
        _service: &crate::server::Service,
        _state: &crate::server::State,
        _model_name: &str,
        _layer_index: u32,
        _neuron_index: u32,
    ) -> Result<serde_json::Value> {
        Ok(json!({}))
    }
}
