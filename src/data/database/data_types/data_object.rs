use std::str::FromStr;

use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use strum::{AsRefStr, EnumDiscriminants, EnumString};

use crate::data::ModelHandle;

#[derive(Clone, Debug, AsRefStr, EnumString, EnumDiscriminants, PartialEq)]
#[strum_discriminants(derive(EnumString, AsRefStr))]
pub enum DataType {
    Neuroscope,
    Neuron2Graph,
    NeuronStore { similarity_threshold: f32 },
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
                Ok(Self::Neuroscope)
            }
            DataTypeDiscriminants::Neuron2Graph => {
                ensure!(
                    type_args.is_empty(),
                    "Neuron2Graph data objects do not take type arguments."
                );
                Ok(Self::Neuron2Graph)
            }
            DataTypeDiscriminants::NeuronStore => {
                ensure!(
                    type_args.len() == 4,
                    "NeuronStore data objects take a single f32 type argument."
                );
                let similarity_threshold: f32 = postcard::from_bytes(type_args).context(
                    "Failed to deserialize f32 similarity threshold for NeuronStore data type.",
                )?;
                Ok(Self::NeuronStore {
                    similarity_threshold,
                })
            }
        }
    }

    pub fn args(&self) -> Vec<u8> {
        match self {
            Self::Neuroscope => Vec::new(),
            Self::Neuron2Graph => Vec::new(),
            Self::NeuronStore {
                similarity_threshold,
            } => postcard::to_allocvec(similarity_threshold).expect("Failed to serialize f32."),
        }
    }
}

#[async_trait]
pub trait ModelDataObject: Sized {
    async fn new(model: &ModelHandle, datatype: DataType) -> Result<Option<Self>>;
    fn data_type() -> DataTypeDiscriminants;
    fn model_handle(&self) -> &ModelHandle;
}
