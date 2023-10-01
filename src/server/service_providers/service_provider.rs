use std::{future::Future, pin::Pin};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use delegate::delegate;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use super::{
    json::Json, metadata::Metadata, neuron2graph::Neuron2Graph,
    neuron2graph_search::Neuron2GraphSearch, neuron_explainer::NeuronExplainer,
    neuroscope::Neuroscope,
};
use crate::{
    data::{DataTypeHandle, Database, ModelHandle},
    server::State,
};

#[allow(unused_variables)]
#[async_trait]
pub trait ServiceProviderTrait: Clone + Serialize + Deserialize<'static> + Send + Sync {
    async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>>;

    async fn model_page(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<serde_json::Value> {
        bail!("No model page exists for service '{}'.", service_name);
    }
    async fn layer_page(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        bail!("No layer page exists for service '{}'.", service_name);
    }
    async fn neuron_page(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        bail!("No neuron page exists for service '{}'.", service_name);
    }
}

#[derive(Clone, Serialize, Deserialize, AsRefStr)]
#[repr(u16)]
pub enum ServiceProvider {
    Metadata = 0,
    Neuroscope = 1,
    NeuronExplainer = 2,
    Neuron2Graph = 3,
    Neuron2GraphSearch = 4,
    Json(Json) = 5,
}

impl ServiceProvider {
    pub fn json(data_type_name: String) -> Self {
        ServiceProvider::Json(Json::new(data_type_name))
    }
}

impl ServiceProvider {
    pub fn is_metadata(&self) -> bool {
        matches!(self, ServiceProvider::Metadata)
    }

    delegate! {
        to match self {
            ServiceProvider::Metadata => Metadata,
            ServiceProvider::Neuroscope => Neuroscope,
            ServiceProvider::NeuronExplainer => NeuronExplainer,
            ServiceProvider::Neuron2Graph => Neuron2Graph,
            ServiceProvider::Neuron2GraphSearch => Neuron2GraphSearch,
            ServiceProvider::Json(json) => json,
        } {
            pub fn required_data_types<'a>(
                &'a self, database: &'a Database,
            ) -> Pin<Box<dyn Future<Output = Result<Vec<DataTypeHandle>>> + Send + 'a>>;

            pub fn model_page<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a>>;

            pub fn layer_page<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
                layer_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a >>;

            pub fn neuron_page<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
                layer_index: u32,
                neuron_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a >>;
        }
    }

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        postcard::to_allocvec(self).with_context(|| {
            format!(
                "Failed to serialize service provider. Type: '{}'",
                self.as_ref()
            )
        })
    }

    pub fn from_binary(bytes: impl AsRef<[u8]>) -> Result<Self> {
        postcard::from_bytes(bytes.as_ref()).context("Failed to deserialize service provider.")
    }
}
