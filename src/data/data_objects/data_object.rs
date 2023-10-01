use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use snap::raw::{Decoder, Encoder};

pub(super) fn to_binary<D: Serialize>(object: &D, object_name: &'static str) -> Result<Vec<u8>> {
    let data = postcard::to_allocvec(object)
        .with_context(|| format!("Failed to serialize {object_name} to binary."))?;
    Encoder::new()
        .compress_vec(data.as_slice())
        .with_context(|| format!("Failed to compress {object_name}."))
}

pub(super) fn from_binary<D: DeserializeOwned + Sized>(
    data: impl AsRef<[u8]>,
    object_name: &'static str,
) -> Result<D> {
    let data = Decoder::new()
        .decompress_vec(data.as_ref())
        .with_context(|| format!("Failed to decompress {object_name}."))?;
    postcard::from_bytes(data.as_slice())
        .with_context(|| format!("Failed to deserialize {object_name} from binary."))
}

pub trait DataObject: Sized + Serialize + DeserializeOwned + Clone {
    fn to_binary(&self) -> Result<Vec<u8>>;

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self>;
}
