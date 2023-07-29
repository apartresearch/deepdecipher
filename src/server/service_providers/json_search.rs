use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    data::{data_types::Json as JsonObject, DataObjectHandle, Database, ModelHandle},
    server::State,
};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonSearch(String);

impl JsonSearch {
    pub fn new(data_object_name: String) -> Self {
        Self(data_object_name)
    }
}

#[async_trait]
impl ServiceProviderTrait for JsonSearch {
    async fn required_data_objects(&self, database: &Database) -> Result<Vec<DataObjectHandle>> {
        let Self(ref data_object_name) = self;
        database.data_object(data_object_name).await?.with_context(|| format!(
            "No data object named '{data_object_name}' in database. This should have been checked when service was created.")
        ).map(|data_object| vec![data_object])
    }

    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        let database = state.database();
        let json_object = database
            .data_object(data_object_name)
            .await
            .with_context(|| {
                format!("Could not get data object '{data_object_name}' from database.")
            })?
            .with_context(|| format!("No data object named '{data_object_name}' in database."))?;
        let json_object: JsonObject = database
            .model_data_object(model, &json_object)
            .await
            .with_context(|| {
                format!(
                    "Failed to get data object '{data_object_name}' for model '{}'.",
                    model.name()
                )
            })?;
        let mut json_data = json_object.model_page().await?;

        let query_key = query
            .as_str()
            .with_context(|| "Query should be a string.")?;

        Ok(json_data.value[query_key].take())
    }
}
