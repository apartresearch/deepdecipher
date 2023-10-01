use std::{collections::HashSet, str::FromStr};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::{
        data_types::NeuronStore as NeuronStoreObject, DataTypeHandle, Database, ModelHandle,
        TokenSearch,
    },
    server::State,
};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2GraphSearch;

#[async_trait]
impl ServiceProviderTrait for Neuron2GraphSearch {
    async fn required_data_types(&self, database: &Database) -> Result<Vec<DataTypeHandle>> {
        database.data_type("neuron_store").await?.context(
            "No data object named 'neuron_store' in database. This should have been checked when service was created."
        ).map(|data_type| vec![data_type])
    }

    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let database = state.database();
        let neuron_store_object = database
            .data_type("neuron_store")
            .await
            .context("Could not get neuron store data object from database.")?
            .context("No data object named 'neuron_store' in database.")?;
        let neuron_store_object: NeuronStoreObject = database
            .model_data_type(model, &neuron_store_object)
            .await
            .with_context(|| {
                format!(
                    "Model '{}' has no 'neuron_store' data object. This should have been checked earlier.",
                    model.name()
                )
            })?;
        let neuron_store = neuron_store_object.get_store().await?;

        let query = query["query"]
            .as_str()
            .context("Query should contain an entry 'query' with a string value.")?;

        let token_searches = query
            .split(',')
            .map(TokenSearch::from_str)
            .collect::<Result<Vec<_>>>()?;
        let results = token_searches
            .into_iter()
            .map(|token_search| {
                let TokenSearch {
                    token,
                    search_types,
                } = token_search;
                search_types
                    .into_iter()
                    .flat_map(|search_type| {
                        neuron_store
                            .get(search_type, token.as_str())
                            .cloned()
                            .unwrap_or_default()
                    })
                    .collect::<HashSet<_>>()
            })
            .reduce(|a, b| a.intersection(&b).copied().collect::<HashSet<_>>())
            .with_context(|| "At least one token search should be provided.")?
            .into_iter()
            .collect::<Vec<_>>();

        Ok(json!(results))
    }
}
