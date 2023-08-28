use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use snap::raw::{Decoder, Encoder};
use utoipa::ToSchema;

use crate::data::NeuronIndex;

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

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let data =
            postcard::to_allocvec(self).context("Failed to serialize neuroscope model page.")?;
        Encoder::new()
            .compress_vec(data.as_slice())
            .context("Failed to compress neuroscope model page.")
    }

    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        let data = Decoder::new()
            .decompress_vec(data.as_ref())
            .context("Failed to decompress neuroscope model page")?;
        postcard::from_bytes(data.as_slice())
            .context("Failed to deserialize neuroscope model page.")
    }
}
