use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ServiceProviderTrait;
use crate::{
    data::{data_objects::MetadataObject, DataTypeHandle, Database, ModelHandle},
    server::State,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata;

#[async_trait]
impl ServiceProviderTrait for Metadata {
    type ModelPageObject = MetadataObject;
    type LayerPageObject = MetadataObject;
    type NeuronPageObject = MetadataObject;

    async fn required_data_types(&self, _database: &Database) -> Result<Vec<DataTypeHandle>> {
        Ok(vec![])
    }

    async fn model_object(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<Self::ModelPageObject> {
        MetadataObject::new(model).await
    }

    async fn layer_object(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        _layer_index: u32,
    ) -> Result<Self::LayerPageObject> {
        MetadataObject::new(model).await
    }

    async fn neuron_object(
        &self,
        _service_name: &str,
        _state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        _layer_index: u32,
        _neuron_index: u32,
    ) -> Result<Self::NeuronPageObject> {
        MetadataObject::new(model).await
    }
}
