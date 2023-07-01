use anyhow::{Context, Result};

use crate::data::{
    database::{Database, ModelHandle},
    NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage,
};

pub struct Neuroscope;

impl Neuroscope {
    pub async fn model_page(
        &self,
        database: &Database,
        model: &ModelHandle,
    ) -> Result<NeuroscopeModelPage> {
        let raw_data = model
            .get_model_data(database, "neuroscope")
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuroscope model data for model '{}'.",
                    model.name()
                )
            })?;
        NeuroscopeModelPage::from_binary(raw_data.as_slice())
    }
    pub async fn layer_page(
        &self,
        database: &Database,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<NeuroscopeLayerPage> {
        let raw_data = model
            .get_layer_data(database, "neuroscope", layer_index)
            .await?;
        NeuroscopeLayerPage::from_binary(raw_data.as_slice())
    }
    pub async fn neuron_page(
        &self,
        database: &Database,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<NeuroscopeNeuronPage> {
        let raw_data = model
            .get_neuron_data(database, "neuroscope", layer_index, neuron_index)
            .await?;
        NeuroscopeNeuronPage::from_binary(raw_data.as_slice())
    }
}
