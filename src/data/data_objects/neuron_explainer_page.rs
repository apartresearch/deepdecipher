use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{data_object, DataObject};

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuronExplainerPage {
    explanation: String,
    score: f32,
}

impl NeuronExplainerPage {
    pub fn from_json(explainer_json: serde_json::Value) -> Result<Self> {
        let scored_explanations_array = explainer_json
            .get("scored_explanations")
            .context("Neuron explainer JSON missing 'scored_explanations' object.")?;

        let scored_explanation_object = scored_explanations_array
            .get(0)
            .context("'scored_explanations' array must have at least one element.")?;
        let explanation_object = scored_explanation_object
            .get("explanation")
            .context("Explanation object missing 'explanation' field.")?;
        let simulation_object = scored_explanation_object
            .get("scored_simulation")
            .context("Explanation object missing 'scored_simulation' object.")?;
        let score_object = simulation_object
            .get("ev_correlation_score")
            .context("Missing 'ev_correlation_score' field in simulation object.")?;

        let explanation = explanation_object
            .as_str()
            .context("'explanation' field should be a string.")?
            .to_owned();
        let score = score_object
            .as_f64()
            .context("'ev_correlation_score' field should be a float.")? as f32;

        Ok(Self { explanation, score })
    }
}

impl DataObject for NeuronExplainerPage {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self)
            .context("Failed to serialize Neuron Explainer neuron page to binary data.")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data)
            .context("Failed to deserialize Neuron Explainer neuron page from binary data.")
    }
}
