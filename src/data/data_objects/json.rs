use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{data_object, DataObject};

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonData {
    pub value: serde_json::Value,
}

impl JsonData {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }
}

impl DataObject for JsonData {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "JSON object")
    }

    fn from_binary(bytes: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(bytes, "JSON object")
    }
}
