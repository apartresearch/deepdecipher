use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use snap::raw::{Decoder, Encoder};

use crate::data::NeuronIndex;

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

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let data =
            postcard::to_allocvec(self).context("Failed to serialize neuroscope layer page.")?;
        Encoder::new()
            .compress_vec(data.as_slice())
            .context("Failed to compress neuroscope layer page.")
    }

    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        let data = Decoder::new()
            .decompress_vec(data.as_ref())
            .context("Failed to decompress neuroscope layer page")?;
        postcard::from_bytes(data.as_slice())
            .context("Failed to deserialize neuroscope layer page.")
    }
}
