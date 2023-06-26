use actix_web::web;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ServiceProvider, State};

#[derive(Clone, Serialize, Deserialize)]
pub struct Service {
    name: String,
    provider: ServiceProvider,
}

impl Service {
    pub fn new(name: String, provider: ServiceProvider) -> Self {
        Self { name, provider }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn model_page(
        &self,
        state: &State,
        query: web::Query<serde_json::Value>,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let service_json = self
            .provider
            .model_page(self, state, query.clone(), model_name)
            .await;
        if self.provider.is_metadata() {
            service_json
        } else {
            let service_json = service_json?;
            let metadata = ServiceProvider::Metadata
                .model_page(self, state, query, model_name)
                .await
                .unwrap_or(serde_json::Value::Null);
            Ok(json!({
                "metadata": metadata,
                "data": service_json
            }))
        }
    }

    pub async fn layer_page(
        &self,
        state: &State,
        query: web::Query<serde_json::Value>,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let service_json = self
            .provider
            .layer_page(self, state, query.clone(), model_name, layer_index)
            .await;
        if self.provider.is_metadata() {
            service_json
        } else {
            let service_json = service_json?;
            let metadata = ServiceProvider::Metadata
                .layer_page(self, state, query, model_name, layer_index)
                .await
                .unwrap_or(serde_json::Value::Null);
            Ok(json!({
                "metadata": metadata,
                "data": service_json
            }))
        }
    }

    pub async fn neuron_page(
        &self,
        state: &State,
        query: web::Query<serde_json::Value>,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let service_json = self
            .provider
            .neuron_page(
                self,
                state,
                query.clone(),
                model_name,
                layer_index,
                neuron_index,
            )
            .await;
        if self.provider.is_metadata() {
            service_json
        } else {
            let service_json = service_json?;
            let metadata = ServiceProvider::Metadata
                .neuron_page(self, state, query, model_name, layer_index, neuron_index)
                .await
                .unwrap_or(serde_json::Value::Null);
            Ok(json!({
                "metadata": metadata,
                "data": service_json
            }))
        }
    }
}
