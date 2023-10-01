use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{
    data_objects::NeuroscopeLayerPage,
    data_objects::NeuroscopeNeuronPage,
    data_objects::{DataObject, NeuroscopeModelPage},
    database::ModelHandle,
    DataTypeHandle,
};

use super::{
    data_type::{DataValidationError, ModelDataType},
    DataTypeDiscriminants,
};

pub struct Neuroscope {
    model: ModelHandle,
    data_type: DataTypeHandle,
}

#[async_trait]
impl ModelDataType for Neuroscope {
    async fn new(model: ModelHandle, data_type: DataTypeHandle) -> Result<Option<Self>> {
        match data_type.data_type().into() {
            DataTypeDiscriminants::Neuroscope => Ok(Some(Self { model, data_type })),
            _ => bail!("Invalid type for Neuroscope data object.",),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Neuroscope
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }

    async fn validate(&self) -> anyhow::Result<Result<(), DataValidationError>> {
        let missing_items: Vec<_> = self.model.missing_items(&self.data_type).await?.collect();
        Ok(if missing_items.is_empty() {
            Ok(())
        } else {
            Err(DataValidationError::MissingItems { missing_items })
        })
    }
}

impl Neuroscope {
    pub async fn model_page(&self) -> Result<NeuroscopeModelPage> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .model_data(&self.data_type)
            .await
            .with_context(|| {
                format!("Failed to get neuroscope model data for model '{model_name}'.",)
            })?
            .with_context(|| {
                format!("Database has no neuroscope model data for model '{model_name}'.")
            })?;
        NeuroscopeModelPage::from_binary(raw_data.as_slice())
    }
    pub async fn layer_page(&self, layer_index: u32) -> Result<NeuroscopeLayerPage> {
        let model_name = self.model.name();
        let raw_data = self.model
            .layer_data( &self.data_type, layer_index)
            .await.with_context(|| {
                format!("Failed to get neuroscope layer data for layer {layer_index} in model '{model_name}'.")
            })?
            .with_context(|| {
                format!("Database has no neuroscope layer data for layer {layer_index} in model '{model_name}'.")
            })?;
        NeuroscopeLayerPage::from_binary(raw_data.as_slice())
    }
    pub async fn neuron_page(
        &self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<NeuroscopeNeuronPage>> {
        let model_name = self.model.name();
        let raw_data = self.model
            .neuron_data( &self.data_type, layer_index, neuron_index)
            .await.with_context(|| {
                format!("Failed to get neuroscope neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        raw_data.map(|raw_data| NeuroscopeNeuronPage::from_binary(raw_data.as_slice())
            .with_context(|| {
                format!("Failed to deserialize neuroscope neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })).transpose()
    }
}
