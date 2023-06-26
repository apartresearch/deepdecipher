use std::{collections::HashSet, str::FromStr};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{data::TokenSearch, server::State};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2GraphSearch;

#[async_trait]
impl ServiceProviderTrait for Neuron2GraphSearch {
    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let query = query["query"]
            .as_str()
            .context("Query should contain an entry 'query' with a string value.")?;
        let neuron_store = state.neuron_store(model_name).await?;
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
