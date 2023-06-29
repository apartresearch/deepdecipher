use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{ServiceProvider, State};

#[derive(Clone, Serialize, Deserialize)]
pub struct Service {
    name: String,
    provider: ServiceProvider,
}

impl Service {
    pub fn new(name: String, provider: ServiceProvider) -> Self {
        assert_ne!(name, "all", "Service name cannot be 'all'.");
        Self { name, provider }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_metadata(&self) -> bool {
        false
    }

    pub async fn model_page(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        self.provider
            .model_page(self.name(), state, query, model_name)
            .await
    }

    pub async fn layer_page(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .layer_page(self.name(), state, query, model_name, layer_index)
            .await
    }

    pub async fn neuron_page(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .neuron_page(
                self.name(),
                state,
                query,
                model_name,
                layer_index,
                neuron_index,
            )
            .await
    }
}
