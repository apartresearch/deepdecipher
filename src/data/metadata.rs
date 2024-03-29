use std::iter;

use serde::{Deserialize, Serialize};

use crate::{data::NeuronIndex, Index};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub num_layers: u32,
    pub layer_size: u32,
    pub activation_function: String,
    pub num_total_neurons: u32,
    pub num_total_parameters: u32,
    pub dataset: String,
}

impl Metadata {
    pub fn neuron_indices(&self) -> impl Iterator<Item = NeuronIndex> {
        let layer_size = self.layer_size;
        (0..self.num_layers).flat_map(move |layer_index| {
            (0..layer_size).map(move |neuron_index| NeuronIndex {
                layer: layer_index,
                neuron: neuron_index,
            })
        })
    }

    pub fn indices(&self) -> impl Iterator<Item = Index> {
        iter::once(Index::Model)
            .chain((0..self.num_layers).map(Index::Layer))
            .chain(self.neuron_indices().map(Index::from))
    }
}
