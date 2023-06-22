use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    path::Path,
};

use actix_web::{get, http::header::ContentType, web, HttpResponse, Responder};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::State;

pub async fn neuron2graph_page(
    model: &str,
    layer_index: u32,
    neuron_index: u32,
) -> Result<serde_json::Value> {
    let path = Path::new("data")
        .join(model)
        .join("neuron2graph")
        .join(format!("layer_{layer_index}",))
        .join(format!("{layer_index}_{neuron_index}"))
        .join("graph");
    fs::read_to_string(path).map(|page| json!(page)).with_context(|| format!("Failed to read neuron2graph page for neuron {neuron_index} in layer {layer_index} of model '{model}'."))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TokenSearchType {
    Activating,
    Important,
}

impl TokenSearchType {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Activating => "activating",
            Self::Important => "important",
        }
    }

    pub fn from_str(s: &str) -> Result<Vec<Self>> {
        match s {
            "activating" => Ok(vec![Self::Activating]),
            "important" => Ok(vec![Self::Important]),
            "any" => Ok(vec![Self::Activating, Self::Important]),
            _ => bail!("Invalid token search type: '{s}'."),
        }
    }
}

impl Display for TokenSearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronStore {
    activating: HashMap<String, HashSet<String>>,
    important: HashMap<String, HashSet<String>>,
}

impl NeuronStore {
    pub fn load(model: &str) -> Result<Self> {
        let neuron_store_path = Path::new("data")
            .join(model)
            .join("neuron2graph-search")
            .join("neuron_store.json");
        let neuron_store_path = neuron_store_path.as_path();
        serde_json::from_reader(
            File::open(neuron_store_path).with_context(|| {
                format!("Could not find neuron store file for model '{model}'.")
            })?,
        )
        .with_context(|| format!("Failed to parse neuron store for model '{model}'."))
    }

    pub fn get(&self, search_type: TokenSearchType, token: &str) -> Option<&HashSet<String>> {
        match search_type {
            TokenSearchType::Activating => self.activating.get(token),
            TokenSearchType::Important => self.important.get(token),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSearch {
    token: String,
    search_types: Vec<TokenSearchType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NeuronIndex {
    layer_index: u32,
    neuron_index: u32,
}

pub async fn neuron2graph_search_page(
    state: &State,
    model: &str,
    query: serde_json::Value,
) -> Result<serde_json::Value> {
    let query = query["query"]
        .as_str()
        .context("Query should contain an entry 'query' with a string value.")?;
    let neuron_store = state.neuron_store(model).await?;
    let token_searches = query
        .split(',')
        .map(|token_search_string| {
            let (search_type_str, token) = token_search_string
                .split(':')
                .collect_tuple()
                .context("Token search string should be of the form 'search_type:token'.")?;
            let search_types = TokenSearchType::from_str(search_type_str)?;
            Ok(TokenSearch {
                token: token.to_string(),
                search_types,
            })
        })
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
        .reduce(|a, b| {
            a.intersection(&b)
                .map(|str| str.to_owned())
                .collect::<HashSet<String>>()
        })
        .with_context(|| "At least one token search should be provided.")?
        .into_iter()
        .map(|neuron_string| {
            let (layer_index, neuron_index) = neuron_string.split('_').collect_tuple().context(
                "Expected all neuron strings to be of the form 'layer_index_neuron_index'.",
            )?;
            Ok(NeuronIndex {
                layer_index: layer_index
                    .parse::<u32>()
                    .with_context(|| format!("Layer index '{layer_index}' not a valid integer"))?,
                neuron_index: neuron_index.parse::<u32>().with_context(|| {
                    format!("Neuron index '{neuron_index}' not a valid integer")
                })?,
            })
        })
        .collect::<Result<Vec<_>>>()
        .with_context(|| "Failed to parse neuron strings into layer and neuron indices.")?;

    Ok(json!(results))
}

#[get("/api/{model}/neuron2graph/{layer_index}/{neuron_index}")]
pub async fn neuron_2_graph(indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();

    match neuron2graph_page(model, layer_index, neuron_index).await {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(page.to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/neuron2graph-search")]
pub async fn neuron2graph_search(
    state: web::Data<State>,
    model: web::Path<String>,
    web::Query(query): web::Query<serde_json::Value>,
) -> impl Responder {
    let model = model.as_str();

    match neuron2graph_search_page(state.as_ref(), model, query).await {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(page.to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}
