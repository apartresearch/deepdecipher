use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub layers: Vec<LayerMetadata>,
    pub activation_function: String,
    pub num_total_neurons: u32,
    pub num_total_parameters: u32,
    pub dataset: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub num_neurons: u32,
}
