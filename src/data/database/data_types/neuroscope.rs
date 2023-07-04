use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::data::{
    database::ModelHandle, DataObjectHandle, NeuroscopeLayerPage, NeuroscopeModelPage,
    NeuroscopeNeuronPage,
};

use super::{data_object::ModelDataObject, DataType, DataTypeDiscriminants};

pub struct Neuroscope {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for Neuroscope {
    async fn new(model: &ModelHandle, datatype: DataType) -> Result<Option<Self>> {
        let data_object = model
            .database()
            .data_object("neuroscope")
            .await?
            .context("No neuroscope data object in database.")?;
        match datatype {
            DataType::Neuroscope => Ok(Some(Self {
                model: model.clone(),
                data_object,
            })),
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
                format!("Database has no neuroscope model data for model '{model_name}'")
            })?;
        NeuroscopeModelPage::from_binary(raw_data.as_slice())
    }
    pub async fn layer_page(&self, layer_index: u32) -> Result<NeuroscopeLayerPage> {
        let model_name = self.model.name();
        let raw_data = self.model
            .layer_data( &self.data_object, layer_index)
            .await?
            .with_context(|| {
                format!("Database has no neuroscope layer data for layer {layer_index} in model '{model_name}'")
            })?;
        NeuroscopeLayerPage::from_binary(raw_data.as_slice())
    }
    pub async fn neuron_page(
        &self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<NeuroscopeNeuronPage> {
        let model_name = self.model.name();
        let raw_data = self.model
            .neuron_data( &self.data_object, layer_index, neuron_index)
            .await?
            .with_context(|| {
                format!("Database has no neuroscope neuron data for neuron n{neuron_index}l{layer_index} in model '{model_name}'")
            })?;
        NeuroscopeNeuronPage::from_binary(raw_data.as_slice())
    }
}
