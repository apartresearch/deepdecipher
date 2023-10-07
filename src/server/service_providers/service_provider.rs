use std::{future::Future, pin::Pin};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use delegate::delegate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::AsRefStr;

use super::{
    json::Json, metadata::Metadata, neuron2graph::Neuron2Graph,
    neuron2graph_search::Neuron2GraphSearch, neuron_explainer::NeuronExplainer,
    neuroscope::Neuroscope,
};
use crate::{
    data::{data_objects::DataObject, DataTypeHandle, Database, ModelHandle},
    server::State,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum NoData {}

impl DataObject for NoData {
    fn to_binary(&self) -> Result<Vec<u8>> {
        unreachable!("NoData can never be instantiated.")
    }

    fn from_binary(_data: impl AsRef<[u8]>) -> Result<Self> {
        unreachable!("NoData can never be instantiated.")
    }
}

#[allow(unused_variables)]
#[async_trait]
pub trait ServiceProviderTrait: Clone + Serialize + Deserialize<'static> + Send + Sync {
    type ModelPageObject: DataObject;
    type LayerPageObject: DataObject;
    type NeuronPageObject: DataObject;

    async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>>;

    async fn model_object(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<Self::ModelPageObject> {
        bail!(
            "No model data exists for service '{}' for model '{}'.",
            service_name,
            model_handle.name()
        );
    }

    async fn layer_object(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<Self::LayerPageObject> {
        bail!(
            "No layer data exists for service '{}' for model '{}'.",
            service_name,
            model_handle.name()
        );
    }

    async fn neuron_object(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Self::NeuronPageObject> {
        bail!(
            "No neuron data exists for service '{}' for model '{}'.",
            service_name,
            model_handle.name()
        );
    }

    async fn model_binary(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<Vec<u8>> {
        self.model_object(service_name, state, query, model_handle)
            .await?
            .to_binary()
    }

    async fn layer_binary(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<Vec<u8>> {
        self.layer_object(service_name, state, query, model_handle, layer_index)
            .await?
            .to_binary()
    }

    async fn neuron_binary(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Vec<u8>> {
        self.neuron_object(
            service_name,
            state,
            query,
            model_handle,
            layer_index,
            neuron_index,
        )
        .await?
        .to_binary()
    }

    async fn model_json(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
    ) -> Result<serde_json::Value> {
        self.model_object(service_name, state, query, model_handle)
            .await
            .map(|object| json!(object))
    }

    async fn layer_json(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        self.layer_object(service_name, state, query, model_handle, layer_index)
            .await
            .map(|object| json!(object))
    }

    async fn neuron_json(
        &self,
        service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_handle: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        self.neuron_object(
            service_name,
            state,
            query,
            model_handle,
            layer_index,
            neuron_index,
        )
        .await
        .map(|object| json!(object))
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

            pub fn model_binary<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
            ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

            pub fn layer_binary<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
                layer_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a >>;

            pub fn neuron_binary<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
                layer_index: u32,
                neuron_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a >>;

            pub fn model_json<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a>>;

            pub fn layer_json<'a>(
                &'a self,
                service_name: &'a str,
                state: &'a State,
                query: &'a serde_json::Value,
                model_handle: &'a ModelHandle,
                layer_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a >>;

            pub fn neuron_json<'a>(
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
