use anyhow::{Context, Result};
use half::f16;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snap::raw::Encoder;

use crate::data::NeuronIndex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Activations {
    neuron_id: NeuronIndex,
    random_sample: Vec<ActivationRecord>,
    random_sample_by_quantile: [Vec<ActivationRecord>; 5],
    quantile_boundaries: [f16; 4],
    mean: f32,
    variance: f32,
    skewness: f32,
    kurtosis: f32,
    most_positive_activation_records: Vec<ActivationRecord>,
}

impl Activations {
    pub fn from_json(json: &Value) -> Result<Self> {
        let json = json
            .as_object()
            .context("neuron explainer JSON not an object.")?;
        let index = {
            let index_json = json
                .get("neuron_id")
                .context("No 'neuron_id' element in neuron explainer JSON.")?;
            NeuronIndex {
                layer: index_json
                    .get("layer_index")
                    .context("No 'layer_index' in 'neuron_id' element in neuron explainer JSON.")?
                    .as_u64()
                    .context("'layer_index' not a u64.")? as u32,
                neuron: index_json
                    .get("neuron_index")
                    .context("No 'neuron_index' in 'neuron_id' element in neuron explainer JSON.")?
                    .as_u64()
                    .context("'neuron_index' not a u64.")? as u32,
            }
        };
        let random_sample = json
            .get("random_sample")
            .with_context(|| {
                format!("No 'random_sample' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_array()
            .context("Object 'random_sample' not an array.")?
            .iter()
            .map(ActivationRecord::from_json)
            .collect::<Result<Vec<_>>>()?;
        let random_sample_by_quantile: Vec<Vec<ActivationRecord>> = json
            .get("random_sample_by_quantile").with_context(|| {
                format!("No 'random_sample_by_quantile' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_array()
            .context("Object 'random_sample_by_quantile' not an array.")?
            .iter()
            .map(|quantile_samples| {
                quantile_samples
                    .as_array()
                    .context("Quantile sample not an array.")?
                    .iter()
                    .map(ActivationRecord::from_json)
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let random_sample_by_quantile: [Vec<ActivationRecord>; 5] =
            random_sample_by_quantile.try_into().map_err(|input: Vec<Vec<ActivationRecord>>|  {
                let input_length = input.len();
                anyhow::anyhow!(
                    "Wrong number of quantiles in neuron explainer JSON for neuron {index}. Should be 5, but is {input_length}."
                )
            })?;
        let quantile_boundaries = json
            .get("quantile_boundaries")
            .with_context(|| {
                format!(
                    "No 'quantile_boundaries' element in neuron explainer JSON for neuron {index}."
                )
            })?
            .as_array()
            .context("Object 'quantile_boundaries' not an array.")?
            .iter()
            .filter_map(|f| f.as_f64().map(f16::from_f64))
            .collect::<Vec<_>>()
            .try_into().map_err(|input: Vec<_>| {
                let input_length = input.len();
                anyhow::anyhow!(
                    "Wrong number of quantile boundaries in neuron explainer JSON for neuron {index}. Should be 4, but is {input_length}."
                )
            })?;
        let mean = json
            .get("mean")
            .with_context(|| {
                format!("No 'mean' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_f64()
            .context("'mean' element must be float.")? as f32;
        let variance = json
            .get("variance")
            .with_context(|| {
                format!("No 'variance' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_f64()
            .context("'variance' element must be float.")? as f32;
        let skewness = json
            .get("skewness")
            .with_context(|| {
                format!("No 'skewness' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_f64()
            .context("'skewness' element must be float.")? as f32;
        let kurtosis = json
            .get("kurtosis")
            .with_context(|| {
                format!("No 'kurtosis' element in neuron explainer JSON for neuron {index}.")
            })?
            .as_f64()
            .context("'kurtosis' element must be float.")? as f32;
        let most_positive_activation_records = json.get("most_positive_activation_records").with_context(|| {
            format!("No 'most_positive_activation_records' element in neuron explainer JSON for neuron {index}.")
        })?
            .as_array()
            .with_context(|| format!("'most_positive_activation_records' element not an array for neuron {index}."))?
            .iter()
            .map(ActivationRecord::from_json)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            neuron_id: index,
            random_sample,
            random_sample_by_quantile,
            quantile_boundaries,
            mean,
            variance,
            skewness,
            kurtosis,
            most_positive_activation_records,
        })
    }

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let data =
            postcard::to_allocvec(&self).context("Failed to serialize neuron explainer object.")?;
        Encoder::new()
            .compress_vec(data.as_slice())
            .context("Failed to compress neuron explainer data.")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    value: String,
}

impl Token {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivationRecord {
    tokens: Vec<Token>,
    activations: Vec<f16>,
}

impl ActivationRecord {
    pub fn new(tokens: Vec<Token>, activations: Vec<f16>) -> Self {
        assert_eq!(tokens.len(), activations.len());
        Self {
            tokens,
            activations,
        }
    }

    pub fn from_json(json: &Value) -> Result<Self> {
        let json = json
            .as_object()
            .context("Activation record not an object.")?;
        let tokens = json
            .get("tokens")
            .context("Activation record missing 'tokens' field.")?
            .as_array()
            .context("Object 'tokens' not an array.")?
            .iter()
            .map(|token| {
                Ok(Token::new(
                    token.as_str().context("Token not a string.")?.to_string(),
                ))
            })
            .collect::<Result<Vec<_>>>()
            .context("Error parsing tokens.")?;
        let activations = json
            .get("activations")
            .context("Activation record missing 'activations' field.")?
            .as_array()
            .context("Object 'activations' not an array.")?
            .iter()
            .map(|activation| {
                Ok(f16::from_f64(
                    activation.as_f64().context("Activation not a float.")?,
                ))
            })
            .collect::<Result<Vec<_>>>()
            .context("Error parsing activations.")?;
        Ok(Self::new(tokens, activations))
    }
}
