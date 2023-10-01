use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use snap::raw::{Decoder, Encoder};

pub(super) fn to_binary<D: Serialize>(object: &D) -> Result<Vec<u8>> {
    let data = postcard::to_allocvec(object).context("Failed to serialize data object.")?;
    Encoder::new()
        .compress_vec(data.as_slice())
        .context("Failed to compress data object.")
}

pub(super) fn from_binary<D: DeserializeOwned + Sized>(data: impl AsRef<[u8]>) -> Result<D> {
    let data = Decoder::new()
        .decompress_vec(data.as_ref())
        .context("Failed to decompress data object.")?;
    postcard::from_bytes(data.as_slice())
        .context("Failed to deserialize neuron explainer neuron page.")
}

pub trait DataObject: Sized + Serialize + DeserializeOwned + Clone {
    fn to_binary(&self) -> Result<Vec<u8>>;

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self>;
}
