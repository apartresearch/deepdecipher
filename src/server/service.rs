use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{response::Body, RequestType, ServiceProvider, State};
use crate::{
    data::{DataTypeHandle, Database, ModelHandle},
    Index,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub provider: ServiceProvider,
}

impl Service {
    pub fn new(name: String, provider: ServiceProvider) -> Self {
        assert_ne!(name, "all", "Service name cannot be 'all'.");
        Self { name, provider }
    }

    pub fn metadata() -> Self {
        Self {
            name: "metadata".to_string(),
            provider: ServiceProvider::Metadata,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_metadata(&self) -> bool {
        false
    }

    pub async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>> {
        self.provider.required_data_types(database).await
    }

    pub async fn model_binary(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<Vec<u8>> {
        self.provider
            .model_binary(self.name(), state, query, model_handle)
            .await
    }

    pub async fn layer_binary(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<Vec<u8>> {
        self.provider
            .layer_binary(self.name(), state, query, model_handle, layer_index)
            .await
    }

    pub async fn neuron_binary(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Vec<u8>> {
        self.provider
            .neuron_binary(
                self.name(),
                state,
                query,
                model_handle,
                layer_index,
                neuron_index,
            )
            .await
    }

    pub async fn model_json(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<serde_json::Value> {
        self.provider
            .model_json(self.name(), state, query, model_handle)
            .await
    }

    pub async fn layer_json(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .layer_json(self.name(), state, query, model_handle, layer_index)
            .await
    }

    pub async fn neuron_json(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        self.provider
            .neuron_json(
                self.name(),
                state,
                query,
                model_handle,
                layer_index,
                neuron_index,
            )
            .await
    }

    pub async fn page(
        &self,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        index: Index,
        request_type: RequestType,
    ) -> Result<Body> {
        Ok(match request_type {
            RequestType::Json => Body::Json(match index {
                Index::Model => self.model_json(state, query, model_handle).await?,
                Index::Layer(layer) => self.layer_json(state, query, model_handle, layer).await?,
                Index::Neuron(layer, neuron) => {
                    self.neuron_json(state, query, model_handle, layer, neuron)
                        .await?
                }
            }),
            RequestType::Binary => Body::Binary(match index {
                Index::Model => self.model_binary(state, query, model_handle).await?,
                Index::Layer(layer) => self.layer_binary(state, query, model_handle, layer).await?,
                Index::Neuron(layer, neuron) => {
                    self.neuron_binary(state, query, model_handle, layer, neuron)
                        .await?
                }
            }),
        })
    }
}
