use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::data::SimilarNeurons;

use super::{data_object, DataObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub graph: String,
}

impl DataObject for Graph {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self)
            .context("Failed to serialize Neuron2Graph graph to binary data.")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data)
            .context("Failed to deserialize Neuron2Graph graph from binary data.")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2GraphData {
    neuron2graph: Graph,
    similar_neurons: SimilarNeurons,
}

impl Neuron2GraphData {
    pub fn graph(&self) -> &Graph {
        &self.neuron2graph
    }

    pub fn similar_neurons(&self) -> &SimilarNeurons {
        &self.similar_neurons
    }
}

impl DataObject for Neuron2GraphData {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self)
            .context("Failed to serialize Neuron2Graph data to binary data.")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data)
            .context("Failed to deserialize Neuron2Graph data from binary data.")
    }
}
