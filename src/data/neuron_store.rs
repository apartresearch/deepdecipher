use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::{fmt::Display, str::FromStr};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use ndarray::{s, Array2};
use serde::Deserialize;
use serde::Serialize;

use super::NeuronIndex;

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
pub struct TokenSearch {
    pub token: String,
    pub search_types: Vec<TokenSearchType>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronStoreRaw {
    activating: HashMap<String, HashSet<String>>,
    important: HashMap<String, HashSet<String>>,
}

impl NeuronStoreRaw {
    pub fn load(data_path: &Path, model: &str) -> Result<Self> {
        let neuron_store_path = data_path
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
    pub fn load(data_path: &Path, model: &str) -> Result<Self> {
        let NeuronStoreRaw {
            activating,
            important,
        } = NeuronStoreRaw::load(data_path, model)?;
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
