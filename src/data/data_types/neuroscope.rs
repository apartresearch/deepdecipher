use anyhow::{ensure, Context, Result};

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
        let raw_data = database.get_model_data(model.name(), "neuroscope").await?;
        serde_json::from_slice(raw_data.as_slice())
            .context("Failed to parse neuroscope model page.")
    }
    pub async fn layer_page(
        &self,
        database: &Database,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<NeuroscopeLayerPage> {
        let raw_data = database
            .get_layer_data(model.name(), "neuroscope", layer_index)
            .await?;
        serde_json::from_slice(raw_data.as_slice())
            .context("Failed to parse neuroscope layer page.")
    }
    pub async fn neuron_page(
        &self,
        database: &Database,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<NeuroscopeNeuronPage> {
        let raw_data = database
            .get_neuron_data(model.name(), "neuroscope", layer_index, neuron_index)
            .await?;
        serde_json::from_slice(raw_data.as_slice())
            .context("Failed to parse neuroscope neuron page.")
    }
}
