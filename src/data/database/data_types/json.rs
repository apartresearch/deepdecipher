use async_trait::async_trait;

use crate::data::{json::JsonData, DataObjectHandle, ModelHandle};

use super::{DataTypeDiscriminants, ModelDataObject};

use anyhow::{bail, Context, Result};

pub struct Json {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for Json {
    async fn new(model: ModelHandle, data_object: DataObjectHandle) -> Result<Option<Self>> {
        match data_object.data_type().into() {
            DataTypeDiscriminants::NeuronStore => Ok(Some(Self { model, data_object })),
            _ => bail!("Invalid type for JSON data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Json
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }
}

impl Json {
    pub async fn model_page(&self) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_object_name = self.data_object.name();
        let raw_data = self
            .model
            .model_data(&self.data_object)
            .await
            .with_context(|| {
                format!("Failed to get '{data_object_name}' model data for model '{model_name}'.",)
            })?
            .with_context(|| {
                format!("Database has no '{data_object_name}' model data for model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
    pub async fn layer_page(&self, layer_index: u32) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_object_name = self.data_object.name();
        let raw_data = self.model
            .layer_data( &self.data_object, layer_index)
            .await.with_context(|| {
                format!("Failed to get '{data_object_name}' layer data for layer {layer_index} in model '{model_name}'.")
            })?
            .with_context(|| {
                format!("Database has no '{data_object_name}' layer data for layer {layer_index} in model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
    pub async fn neuron_page(&self, layer_index: u32, neuron_index: u32) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_object_name = self.data_object.name();
        let raw_data = self.model
            .neuron_data( &self.data_object, layer_index, neuron_index)
            .await.with_context(|| {
                format!("Failed to get '{data_object_name}' neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?
            .with_context(|| {
                format!("Database has no '{data_object_name}' neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
}
