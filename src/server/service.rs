use anyhow::Result;

use super::{ServiceProvider, State};

pub struct Service {
    name: String,
    provider: ServiceProvider,
}

impl Service {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn model_page(&self, state: &State, model_name: &str) -> Result<serde_json::Value> {
        self.provider.model_page(self, state, model_name)
    }

    pub fn layer_page(
        &self,
        state: &State,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .layer_page(self, state, model_name, layer_index)
    }

    pub fn neuron_page(
        &self,
        state: &State,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .neuron_page(self, state, model_name, layer_index, neuron_index)
    }
}
