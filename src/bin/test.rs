use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use deepdecipher::data::NeuronIndex;
use reqwest::Client;
use serde::Serialize;
use snap::raw::Encoder;

fn activations_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/gpt2_small_data/collated-activations/{layer}/{neuron}.json")
}

fn explanations_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/gpt2_small_data/explanations/{layer}/{neuron}.jsonl")
}

fn related_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/gpt2_small_data/related-neurons/weight-based/{layer}/{neuron}.json")
}

async fn fetch_json(url: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let res = client.get(url).send().await?;
    println!("response: {res:?}");
    let page = res.json::<serde_json::Value>().await?;
    Ok(page)
}

async fn activations_page(index: NeuronIndex) -> Result<serde_json::Value> {
    let url = activations_url(index);
    fetch_json(url.as_str()).await
}

fn path(index: NeuronIndex, file_name: &str) -> PathBuf {
    let NeuronIndex { layer, neuron } = index;
    PathBuf::new()
        .join("data")
        .join(format!("{layer:0>2}-{neuron:0>4}-{file_name}"))
}

fn write(index: NeuronIndex, file_name: &str, object: impl AsRef<[u8]>) -> Result<()> {
    let path = path(index, file_name);
    fs::create_dir_all(path.parent().context("No parent")?)?;
    fs::write(path, object)?;
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let index = NeuronIndex {
        layer: 5,
        neuron: 0,
    };
    let start = Instant::now();
    let page = activations_page(index).await?;
    let fetch_time = Instant::now() - start;

    let explanations_page = fetch_json(explanations_url(index).as_str()).await?;
    write(
        index,
        "raw_explanations.json",
        explanations_page.to_string(),
    )?;

    let start = Instant::now();
    let postcard_page = postcard::to_allocvec(&page)?;
    let postcard_page_time = Instant::now() - start;

    let start = Instant::now();
    let compressed_page = Encoder::new().compress_vec(postcard_page.as_slice())?;
    let compressed_page_time = Instant::now() - start;

    println!("Fetch time: {:?}", fetch_time);
    println!("Postcard page time: {:?}", postcard_page_time);
    println!("Compressed page time: {:?}", compressed_page_time);

    write(index, "raw_json.json", page.to_string())?;
    write(index, "raw_json.postcard", postcard_page)?;
    write(index, "compressed_json.postcard", compressed_page)?;

    Ok(())
}
