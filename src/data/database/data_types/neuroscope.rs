use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{
    database::ModelHandle, DataObjectHandle, NeuroscopeLayerPage, NeuroscopeModelPage,
    NeuroscopeNeuronPage,
};

use super::{data_object::ModelDataObject, DataTypeDiscriminants};

pub struct Neuroscope {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for Neuroscope {
    async fn new(model: ModelHandle, data_object: DataObjectHandle) -> Result<Option<Self>> {
        match data_object.data_type().into() {
            DataTypeDiscriminants::Neuroscope => Ok(Some(Self { model, data_object })),
            _ => bail!("Invalid type for Neuroscope data object.",),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Neuroscope
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }
}

impl Neuroscope {
    pub async fn model_page(&self) -> Result<NeuroscopeModelPage> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .model_data(&self.data_object)
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
            .layer_data( &self.data_object, layer_index)
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
            .neuron_data( &self.data_object, layer_index, neuron_index)
            .await.with_context(|| {
                format!("Failed to get neuroscope neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        raw_data.map(|raw_data| NeuroscopeNeuronPage::from_binary(raw_data.as_slice())
            .with_context(|| {
                format!("Failed to deserialize neuroscope neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })).transpose()
    }
}
