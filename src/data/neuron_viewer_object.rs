use std::{fs, path::Path};

use anyhow::{Context, Result};

use half::f16;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuronViewerObject {
    neuron_id: NeuronId,
    random_sample: Vec<ActivationRecord>,
    random_sample_by_quantile: [Vec<ActivationRecord>; 5],
    quantile_boundaries: [f16; 4],
    mean: f32,
    variance: f32,
    skewness: f32,
    kurtosis: f32,
    most_positive_activation_records: Vec<ActivationRecord>,
}

impl NeuronViewerObject {
    pub fn from_json(json: &Value) -> Result<Self> {
        let json = json.as_object().unwrap();
        let neuron_id = NeuronId::from_json(&json["neuron_id"]);
        let random_sample = json["random_sample"]
            .as_array()
            .context("object 'random_sample' not an array")?
            .iter()
            .map(ActivationRecord::from_json)
            .collect::<Result<Vec<_>>>()?;
        let random_sample_by_quantile: Vec<Vec<ActivationRecord>> = json
            ["random_sample_by_quantile"]
            .as_array()
            .context("object 'random_sample_by_quantile' not an array")?
            .iter()
            .map(|quantile_samples| {
                quantile_samples
                    .as_array()
                    .context("quantile sample not an array")?
                    .iter()
                    .map(ActivationRecord::from_json)
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let random_sample_by_quantile: [Vec<ActivationRecord>; 5] =
            random_sample_by_quantile.try_into().unwrap();
        let quantile_boundaries = json["quantile_boundaries"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|f| f.as_f64().map(f16::from_f64))
            .collect::<Vec<_>>()
            .try_into()
            .expect("wrong number of quantile boundaries. Should be 4");
        let mean = json["mean"].as_f64().unwrap() as f32;
        let variance = json["variance"].as_f64().unwrap() as f32;
        let skewness = json["skewness"].as_f64().unwrap() as f32;
        let kurtosis = json["kurtosis"].as_f64().unwrap() as f32;
        let most_positive_activation_records = json["most_positive_activation_records"]
            .as_array()
            .unwrap()
            .iter()
            .map(ActivationRecord::from_json)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            neuron_id,
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

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let data = postcard::to_allocvec(&self).unwrap();
        std::fs::write(path, data).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    token_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuronId {
    layer_index: u32,
    neuron_index: u32,
}

impl NeuronId {
    pub fn from_json(json: &Value) -> Self {
        let json = json.as_object().unwrap();
        let layer_index = json["layer_index"].as_u64().unwrap() as u32;
        let neuron_index = json["neuron_index"].as_u64().unwrap() as u32;
        Self {
            layer_index,
            neuron_index,
        }
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
            .context("activation record not an object")?;
        let tokens = json["tokens"]
            .as_array()
            .context("object 'tokens' not an array")?
            .iter()
            .map(|token| {
                Ok(Token {
                    token_id: token.as_str().context("token not an integer")?.to_string(),
                })
            })
            .collect::<Result<Vec<_>>>()
            .context("error parsing tokens")?;
        let activations = json["activations"]
            .as_array()
            .context("object 'activations' not an array")?
            .iter()
            .map(|activation| {
                Ok(f16::from_f64(
                    activation.as_f64().context("activation not a float")?,
                ))
            })
            .collect::<Result<Vec<_>>>()
            .context("error parsing activations")?;
        Ok(Self::new(tokens, activations))
    }
}
