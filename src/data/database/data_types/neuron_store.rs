use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{DataObjectHandle, ModelHandle, NeuronIndex, NeuronStore as NeuronStoreData};

use super::{DataType, DataTypeDiscriminants, ModelDataObject};

pub struct NeuronStore {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for NeuronStore {
    async fn new(model: &ModelHandle, datatype: DataType) -> Result<Option<Self>> {
        let data_object = model
            .database()
            .data_object("neuron_store")
            .await?
            .context("No neuron store data object in database.")?;
        match datatype {
            DataType::Neuron2Graph => Ok(Some(Self {
                model: model.clone(),
                data_object,
            })),
            _ => bail!("Invalid type for neuron store data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::NeuronStore
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }
}

impl NeuronStore {
    pub async fn get_store(&self) -> Result<NeuronStoreData> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .model_data(&self.data_object)
            .await
            .with_context(|| format!("Failed to get neuron store data for model '{model_name}'.",))?
            .with_context(|| {
                format!("Database has no neuron store data for model '{model_name}'")
            })?;
        NeuronStoreData::from_binary(raw_data)
    }

    pub async fn neuron_similarities(
        &self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Vec<(NeuronIndex, f32)>> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .neuron_data(&self.data_object, layer_index, neuron_index)
            .await
            .with_context(|| format!("Failed to get neuron store data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.",))?
            .with_context(|| {
                format!("Database has no neuron store data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        postcard::from_bytes(raw_data.as_slice()).with_context(|| format!("Failed to deserialize neuron similarities for neuron l{layer_index}n{neuron_index} in model '{model_name}'."))
    }
}
