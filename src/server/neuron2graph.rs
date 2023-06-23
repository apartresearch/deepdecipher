use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    path::Path,
    str::FromStr,
};

use actix_web::{get, http::header::ContentType, web, HttpResponse, Responder};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use ndarray::{s, Array2};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{data::NeuronIndex, server::metadata};

use super::State;

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

    pub fn list_from_str(s: &str) -> Result<Vec<Self>> {
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
pub struct NeuronStoreRaw {
    activating: HashMap<String, HashSet<String>>,
    important: HashMap<String, HashSet<String>>,
}

impl NeuronStoreRaw {
    pub fn load(model: &str) -> Result<Self> {
        let neuron_store_path = Path::new("data")
            .join(model)
            .join("neuron2graph-search")
            .join("neuron_store.json");
        let neuron_store_path = neuron_store_path.as_path();
        let neuron_store_string = fs::read_to_string(neuron_store_path)
            .with_context(|| format!("Could not find neuron store file for model '{model}'."))?;

        serde_json::from_str(&neuron_store_string)
            .with_context(|| format!("Failed to parse neuron store for model '{model}'."))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronStore {
    layer_size: u32,
    num_layers: u32,
    activating: HashMap<String, HashSet<NeuronIndex>>,
    important: HashMap<String, HashSet<NeuronIndex>>,
    related_neurons: Array2<u32>,
}

impl NeuronStore {
    pub fn load(model: &str) -> Result<Self> {
        let NeuronStoreRaw {
            activating,
            important,
        } = NeuronStoreRaw::load(model)?;
        let activating = activating
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    key,
                    value
                        .iter()
                        .map(String::as_str)
                        .map(NeuronIndex::from_str)
                        .collect::<Result<HashSet<_>>>()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let important = important
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    key,
                    value
                        .iter()
                        .map(String::as_str)
                        .map(NeuronIndex::from_str)
                        .collect::<Result<HashSet<_>>>()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let layer_size = 3072;
        let num_layers = 6;
        let num_neurons = (layer_size * num_layers) as usize;
        let mut related_neurons: Array2<u32> = Array2::zeros((num_neurons, num_neurons));
        for (_token, neuron_indices) in activating.iter().chain(important.iter()) {
            for &neuron_index in neuron_indices {
                let index1 = neuron_index.flat_index(layer_size);
                for &other_neuron_index in neuron_indices {
                    let index2 = other_neuron_index.flat_index(layer_size);
                    related_neurons[[index1, index2]] += 1;
                }
            }
        }

        Ok(Self {
            layer_size,
            num_layers,
            activating,
            important,
            related_neurons,
        })
    }

    pub fn similarity(&self, neuron_index1: NeuronIndex, neuron_index2: NeuronIndex) -> f32 {
        let index1 = neuron_index1.flat_index(self.layer_size);
        let self_count1 = self.related_neurons[[index1, index1]];
        let index2 = neuron_index2.flat_index(self.layer_size);
        let self_count2 = self.related_neurons[[index2, index2]];
        (self.related_neurons[[index1, index2]] as f32)
            / (self_count1.max(self_count2).max(1) as f32)
    }

    pub fn similarity_matrix(&self) -> Array2<f32> {
        let num_neurons = (self.layer_size * self.num_layers) as usize;
        let mut matrix = Array2::zeros((num_neurons, num_neurons));
        for i in 0..(self.layer_size * 6) as usize {
            let self_count1 = self.related_neurons[[i, i]];
            for j in 0..(self.layer_size * 6) as usize {
                let self_count2 = self.related_neurons[[j, j]];
                matrix[[i, j]] = (self.related_neurons[[i, j]] as f32)
                    / (self_count1.max(self_count2).max(1) as f32);
            }
        }
        matrix
    }

    pub fn similar_neurons(
        &self,
        neuron_index: NeuronIndex,
        threshold: f32,
    ) -> Result<Vec<(NeuronIndex, f32)>> {
        let index = neuron_index.flat_index(self.layer_size);
        let related_neurons = self.related_neurons.slice(s![index, ..]);
        let self_count1 = *self.related_neurons.get([index, index]).unwrap();
        let mut similar_neurons: Vec<_> = related_neurons
            .iter()
            .copied()
            .enumerate()
            .map(|(index2, common_token_count)| {
                let self_count2 = self.related_neurons[[index2, index2]];
                (
                    index2,
                    (common_token_count as f32) / (self_count1.max(self_count2) as f32),
                )
            })
            .filter(|&(index2, similarity)| index2 != index && similarity >= threshold)
            .map(|(index2, similarity)| {
                (
                    NeuronIndex::from_flat_index(self.layer_size, index2),
                    similarity,
                )
            })
            .collect();
        similar_neurons.sort_by(|(_, similarity1), (_, similarity2)| {
            similarity2
                .partial_cmp(similarity1)
                .unwrap_or(Ordering::Equal)
        });
        Ok(similar_neurons)
    }

    pub fn get(&self, search_type: TokenSearchType, token: &str) -> Option<&HashSet<NeuronIndex>> {
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

impl FromStr for TokenSearch {
    type Err = anyhow::Error;

    fn from_str(token_search_string: &str) -> Result<Self> {
        let (search_type_str, token) = token_search_string
            .split(':')
            .collect_tuple()
            .context("Token search string should be of the form 'search_type:token'.")?;
        let search_types = TokenSearchType::list_from_str(search_type_str)?;
        Ok(TokenSearch {
            token: token.to_string(),
            search_types,
        })
    }
}

pub async fn neuron2graph_page(
    state: &State,
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
    let graph = fs::read_to_string(path).map(|page| json!(page)).with_context(|| format!("Failed to read neuron2graph page for neuron {neuron_index} in layer {layer_index} of model '{model}'."))?;
    let similar_neurons = state
        .neuron_store(model)
        .await?
        .similar_neurons(
            NeuronIndex {
                layer: layer_index,
                neuron: neuron_index,
            },
            0.4,
        )?
        .into_iter()
        .map(
            |(
                NeuronIndex {
                    layer: layer_index,
                    neuron: neuron_index,
                },
                similarity,
            )| {
                json!({
                    "layer": layer_index,
                    "neuron": neuron_index,
                    "similarity": similarity,
                })
            },
        )
        .collect::<Vec<_>>();
    Ok(json!({
        "graph": graph,
        "similar": similar_neurons,}))
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

#[get("/api/{model}/neuron2graph/{layer_index}/{neuron_index}")]
pub async fn neuron_2_graph(
    state: web::Data<State>,
    indices: web::Path<(String, u32, u32)>,
) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model_name = model.as_str();
    let model_metadata = metadata::model_page(model_name).unwrap_or_else(|_| json!(null));

    match neuron2graph_page(state.as_ref(), model_name, layer_index, neuron_index).await {
        Ok(page) => HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(&json!({"model": model_metadata, "neuron2graph": page}))
                .expect("Failed to serialize page to JSON. This should always be possible."),
        ),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/neuron2graph-search")]
pub async fn neuron2graph_search(
    state: web::Data<State>,
    model: web::Path<String>,
    web::Query(query): web::Query<serde_json::Value>,
) -> impl Responder {
    let model_name = model.as_str();
    let model_metadata = metadata::model_page(model_name).unwrap_or_else(|_| json!(null));

    match neuron2graph_search_page(state.as_ref(), model_name, query).await {
        Ok(page) => HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(&json!({"model": model_metadata, "neuron2graph": page}))
                .expect("Failed to serialize page to JSON. This should always be possible."),
        ),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}
