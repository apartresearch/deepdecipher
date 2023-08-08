use anyhow::Result;
use deepdecipher::data::{data_types::Neuroscope, Database};

#[tokio::main]
pub async fn main() -> Result<()> {
    let database = Database::open("data.db").await?;
    let model = database.model("solu-1l").await?.unwrap();
    let data_object = database.data_object("neuroscope").await?.unwrap();
    let data: Neuroscope = model.data_object(&data_object).await?;
    let page = data.neuron_page(0, 0).await?.unwrap();
    let token_lengths: Vec<_> = page
        .texts()
        .iter()
        .flat_map(|text| text.tokens().iter())
        .map(String::len)
        .collect();
    let avg_length = token_lengths.iter().sum::<usize>() as f64 / token_lengths.len() as f64;
    println!("avg length: {}", avg_length);
    Ok(())
}
