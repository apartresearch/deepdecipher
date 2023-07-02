use std::str::FromStr;

use anyhow::{ensure, Context, Result};
use strum::{AsRefStr, EnumDiscriminants, EnumString};

use crate::data::ModelHandle;

#[derive(Clone, Debug, AsRefStr, EnumString, EnumDiscriminants, PartialEq, Eq)]
#[strum_discriminants(derive(EnumString, AsRefStr))]
pub enum DataType {
    Neuroscope,
}

impl DataType {
    pub fn from_raw(data_type: &str, type_args: &[u8]) -> Result<Self> {
        match DataTypeDiscriminants::from_str(data_type)
            .with_context(|| format!("Unexpected data type '{data_type}'."))?
        {
            DataTypeDiscriminants::Neuroscope => {
                ensure!(
                    type_args.is_empty(),
                    "Neuroscope data objects do not take type arguments."
                );
                Ok(DataType::Neuroscope)
            }
        }
    }

    pub fn args(&self) -> Vec<u8> {
        match self {
            Self::Neuroscope => Vec::new(),
        }
    }
}

pub trait ModelDataObject: Sized {
    fn new(model: &ModelHandle, datatype: DataType) -> Result<Option<Self>>;
    fn data_type() -> DataTypeDiscriminants;
    fn model_handle(&self) -> &ModelHandle;
}
