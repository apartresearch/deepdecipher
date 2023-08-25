use anyhow::Result;
use deepdecipher::{
    data::{retrieve::neuroscope::scrape_neuron_page, NeuronIndex},
    server,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let result = scrape_neuron_page(
        "solu-12l",
        NeuronIndex {
            layer: 9,
            neuron: 5596,
        },
    )
    .await?;

    Ok(())
}
