use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::{fmt::Display, str::FromStr};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use snap::raw::{Decoder, Encoder};

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SimilarNeuron {
    layer: u32,
    neuron: u32,
    similarity: f32,
}

impl SimilarNeuron {
    fn new(neuron_index: NeuronIndex, similarity: f32) -> Self {
        Self {
            layer: neuron_index.layer,
            neuron: neuron_index.neuron,
            similarity,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimilarNeurons {
    #[serde(rename = "similar")]
    similar_neurons: Vec<SimilarNeuron>,
}

impl SimilarNeurons {
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let result = postcard::to_allocvec(self.similar_neurons.as_slice())?;
        Ok(result)
    }

    pub fn from_binary(bytes: &[u8]) -> Result<Self> {
        let result = postcard::from_bytes(bytes)?;
        Ok(result)
    }
}

pub struct NeuronSimilarity {
    layer_size: u32,
    similar_neurons: Vec<SimilarNeurons>,
}

impl NeuronSimilarity {
    pub fn similar_neurons(&self, neuron_index: NeuronIndex) -> Result<&SimilarNeurons> {
        let index = neuron_index.flat_index(self.layer_size);
        self.similar_neurons.get(index).with_context(|| format!("No similar neuron array for neuron index {neuron_index}."))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronStoreRaw {
    activating: HashMap<String, HashSet<String>>,
    important: HashMap<String, HashSet<String>>,
}

impl NeuronStoreRaw {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let neuron_store_path = path.as_ref();
        let neuron_store_string = fs::read_to_string(neuron_store_path).with_context(|| {
            format!("Could not read neuron store from '{neuron_store_path:?}'.")
        })?;

        serde_json::from_str(&neuron_store_string)
            .with_context(|| format!("Failed to parse neuron store from '{neuron_store_path:?}'."))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronStore {
    layer_size: u32,
    num_layers: u32,
    activating: HashMap<String, HashSet<NeuronIndex>>,
    important: HashMap<String, HashSet<NeuronIndex>>,
}

impl NeuronStore {
    pub fn neuron_similarity(&self, threshold: f32) -> Result<NeuronSimilarity> {
        let num_neurons = self.num_layers as usize * self.layer_size as usize;

        println!("Finding activating tokens for all neurons...");
        std::io::stdout().flush().unwrap();
        let mut activating_tokens: Vec<Vec<u32>> = (0..num_neurons).map(|_| Vec::new()).collect();
        for (token_id, (_token, neuron_indices)) in self.activating.iter().enumerate() {
            for &neuron_index in neuron_indices {
                let index = neuron_index.flat_index(self.layer_size);
                activating_tokens.get_mut(index).with_context(|| 
                    format!("Index {index} somehow greater than the number of neurons. This should not be possible")
                )?.push(token_id as u32);
            }
        }

        println!("Finding important tokens for all neurons...");
        std::io::stdout().flush().unwrap();
        let mut important_tokens: Vec<Vec<u32>> = (0..num_neurons).map(|_| Vec::new()).collect();
        for (token_id, (_token, neuron_indices)) in self.important.iter().enumerate() {
            for &neuron_index in neuron_indices {
                let index = neuron_index.flat_index(self.layer_size);
                important_tokens.get_mut(index).with_context(|| 
                    format!("Index {index} somehow greater than the number of neurons. This should not be possible")
                )?.push(token_id as u32);
            }
        }

        println!("Finding similar neurons...");
        std::io::stdout().flush().unwrap();
        let mut similar_neurons: Vec<SimilarNeurons> = (0..num_neurons).map(|_| SimilarNeurons { similar_neurons: vec![] }).collect();
        let start = Instant::now();
        for (neuron, (this_activating_tokens, this_important_tokens)) in activating_tokens.iter().zip(important_tokens.iter()).enumerate() {
            let this_neuron_index = NeuronIndex::from_flat_index(self.layer_size, neuron);
            let neurons_per_second = (neuron as f32) / start.elapsed().as_secs_f32();
            print!("Neuron {this_neuron_index}. Neurons per second: {neurons_per_second:.0}        \r");
            for (other_neuron, (other_activating_tokens, other_important_tokens)) in activating_tokens.iter().zip(important_tokens.iter()).enumerate().skip(neuron + 1) {
                let mut total_common = 0;
                
                let mut this_activating_iter = this_activating_tokens.iter().copied().peekable();
                let mut other_activating_iter = other_activating_tokens.iter().copied().peekable();
                while let (Some(this_activating), Some(other_activating)) = (this_activating_iter.peek(), other_activating_iter.peek()) {
                    match this_activating.cmp(other_activating) {
                        std::cmp::Ordering::Less => { this_activating_iter.next(); },
                        std::cmp::Ordering::Equal => {
                            total_common += 1;
                            this_activating_iter.next();
                            other_activating_iter.next();

                        },
                        std::cmp::Ordering::Greater => { other_activating_iter.next(); },
                    }
                }
                let mut this_important_iter = this_important_tokens.iter().copied().peekable();
                let mut other_important_iter = other_important_tokens.iter().copied().peekable();
                while let (Some(this_important), Some(other_important)) = (this_important_iter.peek(), other_important_iter.peek()) {
                    match this_important.cmp(other_important) {
                        std::cmp::Ordering::Less => { this_important_iter.next(); },
                        std::cmp::Ordering::Equal => {
                            total_common += 1;
                            this_important_iter.next();
                            other_important_iter.next();

                        },
                        std::cmp::Ordering::Greater => { other_important_iter.next(); },
                    }
                }

                let possible_common = activating_tokens.len().min(other_activating_tokens.len()) + important_tokens.len().min(other_important_tokens.len());
                let similarity = (total_common as f32) / (possible_common as f32);
                if similarity >= threshold {
                    let other_neuron_index = NeuronIndex::from_flat_index(self.layer_size, other_neuron);
                    similar_neurons.get_mut(neuron).with_context(|| 
                        format!("Index {neuron} of neuron somehow greater than the number of neurons. This should not be possible")
                    )?.similar_neurons.push(SimilarNeuron::new(other_neuron_index, similarity));
                    similar_neurons.get_mut(other_neuron).with_context(|| 
                        format!("Index {other_neuron} of other neuron somehow greater than the number of neurons. This should not be possible")
                    )?.similar_neurons.push(SimilarNeuron::new(this_neuron_index, similarity));
                }
            }
        }

        Ok(NeuronSimilarity {
            layer_size: self.layer_size,
            similar_neurons,
        })
    }

    pub fn from_raw(raw: NeuronStoreRaw, num_layers: u32, layer_size: u32) -> Result<Self> {
        let NeuronStoreRaw {
            activating,
            important,
        } = raw;
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

        Ok(Self {
            layer_size,
            num_layers,
            activating,
            important,
        })
    }

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let data = postcard::to_allocvec(self).context("Failed to serialize neuron store.")?;
        Encoder::new()
            .compress_vec(data.as_slice())
            .context("Failed to compress neuron store.")
    }

    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        let data = Decoder::new()
            .decompress_vec(data.as_ref())
            .context("Failed to decompress neuron store")?;
        postcard::from_bytes(data.as_slice()).context("Failed to deserialize neuron store.")
    }

    pub fn get(&self, search_type: TokenSearchType, token: &str) -> Option<&HashSet<NeuronIndex>> {
        match search_type {
            TokenSearchType::Activating => self.activating.get(token),
            TokenSearchType::Important => self.important.get(token),
        }
    }
}
