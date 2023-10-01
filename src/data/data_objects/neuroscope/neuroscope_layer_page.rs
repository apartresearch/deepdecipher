use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::data::{
    data_objects::{data_object, DataObject},
    NeuronIndex,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuroscopeLayerPage {
    num_neurons: u32,
    important_neurons: Vec<(NeuronIndex, f32)>,
}

impl NeuroscopeLayerPage {
    pub fn new(mut important_neurons: Vec<(NeuronIndex, f32)>) -> Self {
        important_neurons.sort_unstable_by(
            |(_, self_activation_range), (_, other_activation_range)| {
                self_activation_range.total_cmp(other_activation_range)
            },
        );
        Self {
            num_neurons: important_neurons.len() as u32,
            important_neurons,
        }
    }

    pub fn num_neurons(&self) -> u32 {
        self.num_neurons
    }

    pub fn important_neurons(&self) -> &[(NeuronIndex, f32)] {
        self.important_neurons.as_slice()
    }
}

impl DataObject for NeuroscopeLayerPage {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuroscope layer page")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuroscope layer page")
    }
}
