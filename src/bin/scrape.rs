use anyhow::{Context, Result};
use neuronav::data::retrieve;
use tokio::runtime::Runtime;

pub fn main() -> Result<()> {
    let data_path = "data";
    let model = "solu-1l";
    let layer_index = 0;
    let num_neurons = 2048;
    println!("Outside of runtime.");
    Runtime::new()
        .context("Failed to start async runtime to scrape neuroscope.")?
        .block_on(async {
            println!("Inside runtime");
            retrieve::neuroscope::scrape_layer_to_files(data_path, model, layer_index, num_neurons)
                .await
                .context("Failed to scrape layer.")
        })?;
    Ok(())
}
