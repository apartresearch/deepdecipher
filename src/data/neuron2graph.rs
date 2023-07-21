use anyhow::{Context, Result};
use snap::raw::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct Graph {
    pub graph: String,
}

impl Graph {
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let bytes = self.graph.as_bytes().to_vec();
        Encoder::new()
            .compress_vec(bytes.as_slice())
            .context("Failed to compress neuron2graph neuron graph.")
    }

    pub fn from_binary(bytes: impl AsRef<[u8]>) -> Result<Self> {
        let bytes = Decoder::new()
            .decompress_vec(bytes.as_ref())
            .context("Failed to decompress neuron2graph neuron graph")?;
        String::from_utf8(bytes)
            .context("Neuron2Graph graph string is not valid UTF-8.")
            .map(|graph| Self { graph })
    }
}
