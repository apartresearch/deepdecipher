use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{
    neuron_store::SimilarNeurons, DataObjectHandle, ModelHandle, NeuronStore as NeuronStoreData,
};

use super::{data_type::DataValidationError, DataTypeDiscriminants, ModelDataObject};

pub struct NeuronStore {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for NeuronStore {
    async fn new(model: ModelHandle, data_object: DataObjectHandle) -> Result<Option<Self>> {
        match data_object.data_type().into() {
            DataTypeDiscriminants::NeuronStore => Ok(Some(Self { model, data_object })),
            _ => bail!("Invalid type for neuron store data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::NeuronStore
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }

    async fn validate(&self) -> anyhow::Result<Result<(), DataValidationError>> {
        let missing_items: Vec<_> = self
            .model
            .missing_model_items(&self.data_object)
            .await?
            .chain(self.model.missing_neuron_items(&self.data_object).await?)
            .collect();
        Ok(if missing_items.is_empty() {
            Ok(())
        } else {
            Err(DataValidationError::MissingItems { missing_items })
        })
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
    ) -> Result<SimilarNeurons> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .neuron_data(&self.data_object, layer_index, neuron_index)
            .await
            .with_context(|| format!("Failed to get neuron store data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.",))?
            .with_context(|| {
                format!("Database has no neuron store data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        SimilarNeurons::from_binary(raw_data.as_slice()).with_context(|| format!("Failed to deserialize neuron similarities for neuron l{layer_index}n{neuron_index} in model '{model_name}'."))
    }
}
