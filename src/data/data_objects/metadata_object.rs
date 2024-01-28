use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{data_object, DataObject};
use crate::data::{Metadata, ModelHandle};

#[derive(Clone, Serialize, Deserialize)]
pub struct MetadataObject {
    #[serde(flatten)]
    pub metadata: Metadata,
    pub available_services: Vec<String>,
}

impl MetadataObject {
    pub async fn new(model_handle: &ModelHandle) -> Result<Self> {
        let metadata = model_handle.metadata().clone();
        let available_services: Vec<_> = model_handle
            .available_services()
            .await?
            .into_iter()
            .map(|service| service.name().to_owned())
            .collect();
        Ok(MetadataObject {
            metadata,
            available_services,
        })
    }
}

impl DataObject for MetadataObject {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "metadata")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "metadata")
    }
}
