use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::data::data_types::Json as JsonData;
use crate::data::{DataObjectHandle, Database, ModelHandle};
use crate::server::State;

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Json(String);

impl Json {
    pub fn new(data_object_name: String) -> Self {
        Self(data_object_name)
    }
}

async fn data_object(
    database: &Database,
    model: &ModelHandle,
    data_object_name: &str,
) -> Result<JsonData> {
    let data_object = database
        .data_object(data_object_name)
        .await?
        .with_context(|| format!("No data object with name '{data_object_name}'."))?;
    model.data_object(&data_object).await.with_context(|| {
        format!(
            "Failed to get json data object '{data_object_name}' for model '{}'.",
            model.name()
        )
    })
}

#[async_trait]
impl ServiceProviderTrait for Json {
    async fn required_data_objects(&self, database: &Database) -> Result<Vec<DataObjectHandle>> {
        let Self(ref data_object_name) = self;
        let data_object = database
            .data_object(data_object_name)
            .await?
            .with_context(|| format!("No data object with name '{data_object_name}'. This should have been checked when the service was created."))?;
        Ok(vec![data_object])
    }

    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        let page = data_object(&state.database(), model, data_object_name)
            .await?
            .model_page()
            .await?;
        Ok(page.value)
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        let page = data_object(&state.database(), model, data_object_name)
            .await?
            .layer_page(layer_index)
            .await?;
        Ok(page.value)
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        _query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        let page = data_object(&state.database(), model, data_object_name)
            .await?
            .neuron_page(layer_index, neuron_index)
            .await?;
        Ok(page.value)
    }
}
