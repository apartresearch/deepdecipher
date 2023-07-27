use anyhow::{Context, Result};
use snap::raw::{Decoder, Encoder};

pub struct JsonData {
    pub value: serde_json::Value,
}

impl JsonData {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let bytes =
            postcard::to_allocvec(&self.value).context("Failed to serialize JSON value.")?;
        Encoder::new()
            .compress_vec(bytes.as_slice())
            .context("Failed to compress JSON value.")
    }

    pub fn from_binary(bytes: impl AsRef<[u8]>) -> Result<Self> {
        let bytes = Decoder::new()
            .decompress_vec(bytes.as_ref())
            .context("Failed to decompress JSON value.")?;
        postcard::from_bytes(bytes.as_slice())
            .context("Failed to deserialize JSON value.")
            .map(|value| Self { value })
    }
}
