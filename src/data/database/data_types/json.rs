use async_trait::async_trait;

use crate::{
    data::{json::JsonData, DataTypeHandle, ModelHandle},
    Index,
};

use super::{data_type::DataValidationError, DataTypeDiscriminants, ModelDataType};

use anyhow::{bail, Context, Result};

pub struct Json {
    model: ModelHandle,
    data_type: DataTypeHandle,
}

#[async_trait]
impl ModelDataType for Json {
    async fn new(model: ModelHandle, data_type: DataTypeHandle) -> Result<Option<Self>> {
        match data_type.data_type().into() {
            DataTypeDiscriminants::Json => Ok(Some(Self { model, data_type })),
            _ => bail!("Invalid type for JSON data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Json
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }

    async fn validate(&self) -> anyhow::Result<Result<(), DataValidationError>> {
        // We cannot validate JSON data objects since we do not know what they must contain.
        Ok(Ok(()))
    }
}

impl Json {
    pub async fn page(&self, index: Index) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_type_name = self.data_type.name();
        let raw_data = self
            .model
            .data(&self.data_type, index)
            .await
            .with_context(|| {
                format!(
                    "Failed to get '{data_type_name}' data for {index} in model '{model_name}'.",
                    index = index.error_string()
                )
            })?
            .with_context(|| {
                format!(
                    "Database has no '{data_type_name}' data for {index} in model '{model_name}'.",
                    index = index.error_string()
                )
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }

    pub async fn model_page(&self) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_type_name = self.data_type.name();
        let raw_data = self
            .model
            .model_data(&self.data_type)
            .await
            .with_context(|| {
                format!("Failed to get '{data_type_name}' model data for model '{model_name}'.",)
            })?
            .with_context(|| {
                format!("Database has no '{data_type_name}' model data for model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
    pub async fn layer_page(&self, layer_index: u32) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_type_name = self.data_type.name();
        let raw_data = self.model
            .layer_data( &self.data_type, layer_index)
            .await.with_context(|| {
                format!("Failed to get '{data_type_name}' layer data for layer {layer_index} in model '{model_name}'.")
            })?
            .with_context(|| {
                format!("Database has no '{data_type_name}' layer data for layer {layer_index} in model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
    pub async fn neuron_page(&self, layer_index: u32, neuron_index: u32) -> Result<JsonData> {
        let model_name = self.model.name();
        let data_type_name = self.data_type.name();
        let raw_data = self.model
            .neuron_data( &self.data_type, layer_index, neuron_index)
            .await.with_context(|| {
                format!("Failed to get '{data_type_name}' neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?
            .with_context(|| {
                format!("Database has no '{data_type_name}' neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        JsonData::from_binary(raw_data.as_slice())
    }
}
