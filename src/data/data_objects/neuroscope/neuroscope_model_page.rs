use anyhow::Result;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::data::{
    data_objects::{data_object, DataObject},
    NeuronIndex,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NeuroscopeModelPage {
    important_neurons: Vec<(NeuronIndex, f32)>,
}

impl NeuroscopeModelPage {
    pub fn new(mut important_neurons: Vec<(NeuronIndex, f32)>) -> Self {
        important_neurons.sort_unstable_by(|(_, self_importance), (_, other_importance)| {
            self_importance.total_cmp(other_importance)
        });
        Self { important_neurons }
    }

    pub fn important_neurons(&self) -> &[(NeuronIndex, f32)] {
        self.important_neurons.as_slice()
    }
}

impl DataObject for NeuroscopeModelPage {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuroscope model page")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuroscope model page")
    }
}
