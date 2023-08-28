use std::{fmt::Display, str::FromStr};

use anyhow::{Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema,
)]
pub struct NeuronIndex {
    pub layer: u32,
    pub neuron: u32,
}

impl NeuronIndex {
    pub fn from_flat_index(layer_size: u32, flat_index: usize) -> Self {
        let layer_index = flat_index as u32 / layer_size;
        let neuron_index = flat_index as u32 % layer_size;
        Self {
            layer: layer_index,
            neuron: neuron_index,
        }
    }

    pub fn flat_index(&self, layer_size: u32) -> usize {
        (self.layer * layer_size + self.neuron) as usize
    }

    pub fn iter(num_layers: u32, layer_size: u32) -> impl Iterator<Item = Self> {
        (0..num_layers)
            .cartesian_product(0..layer_size)
            .map(|(layer, neuron)| Self { layer, neuron })
    }
}

impl FromStr for NeuronIndex {
    type Err = anyhow::Error;
    fn from_str(neuron_index_string: &str) -> Result<Self> {
        let (layer_index, neuron_index) = neuron_index_string
            .split('_')
            .collect_tuple()
            .context("Expected all neuron strings to be of the form 'layer_index_neuron_index'.")?;
        Ok(NeuronIndex {
            layer: layer_index
                .parse::<u32>()
                .with_context(|| format!("Layer index '{layer_index}' not a valid integer"))?,
            neuron: neuron_index
                .parse::<u32>()
                .with_context(|| format!("Neuron index '{neuron_index}' not a valid integer"))?,
        })
    }
}

impl Display for NeuronIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self { layer, neuron } = self;
        write!(f, "l{layer}n{neuron}")
    }
}
