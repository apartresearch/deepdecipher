use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use flate2::{bufread::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};

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

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        fs::create_dir_all(
            path.parent()
                .with_context(|| format!("Invalid path '{path:?}'"))?,
        )
        .with_context(|| format!("Failed to create directory for '{path:?}'"))?;
        let data = postcard::to_allocvec(&self).context("Failed to serialize neuroscope page.")?;

        let file =
            File::create(path).with_context(|| format!("Failed to create file '{path:?}'."))?;
        let mut encoder = DeflateEncoder::new(file, Compression::default());
        encoder
            .write_all(&data)
            .context("Failed to compress neuroscope page.")
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| format!("Failed to open file '{path:?}'."))?;
        let buf_reader = BufReader::new(file);
        let decoder = DeflateDecoder::new(buf_reader);
        let data = decoder
            .bytes()
            .collect::<io::Result<Vec<u8>>>()
            .context("Failed to decompress neuroscope page.")?;

        postcard::from_bytes(&data)
            .with_context(|| format!("Failed to deserialize neuroscope page from file '{path:?}'."))
    }
}
