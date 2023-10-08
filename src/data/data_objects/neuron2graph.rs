use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::data::SimilarNeurons;

use super::{data_object, DataObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub graph: String,
}

impl DataObject for Graph {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuron2Graph graph")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuron2Graph graph")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2GraphData {
    pub graph: Graph,
    pub similar: SimilarNeurons,
}

impl DataObject for Neuron2GraphData {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuron2Graph data")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuron2Graph data")
    }
}
