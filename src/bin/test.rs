use ndarray::Axis;
use neuronav::server::neuron2graph::NeuronStore;

use anyhow::Result;

pub fn main() -> Result<()> {
    let neuron_store = NeuronStore::load("solu-6l-pile")?;
    let matrix = neuron_store.similarity_matrix();
    for &threshold in &[0.01, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
        let mean_above_threshold = matrix
            .map_axis(Axis(1), |row| {
                row.iter().filter(|&&x| x > threshold).count() as f32
            })
            .mean()
            .unwrap()
            - 1.;
        println!("Mean above {}: {}", threshold, mean_above_threshold);
    }
    Ok(())
}
