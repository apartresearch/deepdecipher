use crate::data::Metadata;
use anyhow::{anyhow, Result};

#[derive(Clone, Copy, Debug)]
pub enum Index {
    Model,
    Layer(u32),
    Neuron(u32, u32),
}

impl Index {
    pub fn model() -> Self {
        Self::Model
    }

    pub fn layer(layer_index: u32) -> Self {
        Self::Layer(layer_index)
    }

    pub fn neuron(layer_index: u32, neuron_index: u32) -> Self {
        Self::Neuron(layer_index, neuron_index)
    }

    pub fn valid_in_model(self, metadata: &Metadata) -> Result<()> {
        let model_name = metadata.name.as_str();
        let num_layers = metadata.num_layers;
        let layer_size = metadata.layer_size;

        match self {
            Self::Layer(layer_index) | Self::Neuron(layer_index, _) if layer_index >= num_layers => Err(anyhow!(
                "Layer index is {layer_index} but model '{model_name}' only has {num_layers} layers."
            )),
            Self::Neuron(_, neuron_index) if neuron_index >= layer_size => Err(anyhow!(
                "Neuron index is {neuron_index} but model '{model_name}' only has {layer_size} neurons per layer."
            )),
            _ => Ok(())
        }
    }

    pub fn error_string(self) -> String {
        match self {
            Self::Model => "model".to_string(),
            Self::Layer(layer_index) => format!("layer {layer_index}"),
            Self::Neuron(layer_index, neuron_index) => {
                format!("neuron l{layer_index}n{neuron_index}")
            }
        }
    }
}
