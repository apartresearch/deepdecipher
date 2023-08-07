use serde::{Deserialize, Serialize};

use crate::data::NeuronIndex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Explanations {
    index: NeuronIndex,
    explanations: Vec<ScoredExplanation>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ScoredExplanation {
    explanation: String,
    scored_simulation: ScoredSimulation,
}

#[derive(Clone, Serialize, Deserialize)]
struct ScoredSimulation {
    simulation: String,
    score: f32,
}
