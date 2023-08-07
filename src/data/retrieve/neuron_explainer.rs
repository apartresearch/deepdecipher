use std::{panic, sync::Arc, time::Instant};

use anyhow::{ensure, Context, Result};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
    data::{
        data_types::DataType, neuron_explainer_page::NeuronExplainerPage, ModelHandle, NeuronIndex,
    },
    util::Progress,
};

const SMALL_NUM_LAYERS: u32 = 12;
const SMALL_LAYER_SIZE: u32 = 3072;
const XL_NUM_LAYERS: u32 = 48;
const XL_LAYER_SIZE: u32 = 6400;

fn small_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/gpt2_small_data/explanations/{layer}/{neuron}.jsonl")
}

fn xl_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/data/explanations/{layer}/{neuron}.jsonl")
}

pub async fn fetch_neuron(url: impl AsRef<str>) -> Result<NeuronExplainerPage> {
    let client = Client::new();
    let res = client.get(url.as_ref()).send().await?;
    let page = res.json::<serde_json::Value>().await?;

    NeuronExplainerPage::from_json(page)
}

async fn fetch(
    num_layers: u32,
    layer_size: u32,
    url: impl Fn(NeuronIndex) -> String,
) -> Result<Vec<NeuronExplainerPage>> {
    let mut join_set = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(20));

    let mut progress = Progress::start((num_layers * layer_size) as u64, "Fetching data");
    progress.print();
    for index in NeuronIndex::iter(num_layers, layer_size) {
        let url = url(index);

        let semaphore = Arc::clone(&semaphore);
        join_set.spawn(async move {
            let permit = semaphore.acquire_owned().await.unwrap();
            let result = fetch_neuron(url).await.with_context(|| {
                format!("Failed to fetch neuron explainer data for neuron {index}.")
            })?;
            drop(permit);
            Ok::<_, anyhow::Error>((index, result))
        });
    }

    let mut result = Vec::with_capacity((num_layers * layer_size) as usize);

    while let Some(join_result) = join_set.join_next().await {
        let output = match join_result {
            Ok(scrape_result) => scrape_result?,
            Err(join_error) => {
                let panic_object = join_error
                    .try_into_panic()
                    .expect("Should be impossible to cancel these tasks.");
                panic::resume_unwind(panic_object);
            }
        };
        result.push(output);
        progress.increment();
        progress.print();
    }

    println!("Data fetched.                                          ");
    result.sort_by_key(|(index, _)| *index);
    Ok(result.into_iter().map(|(_, data)| data).collect())
}

async fn fetch_to_database(
    model_handle: &mut ModelHandle,
    url: impl Fn(NeuronIndex) -> String,
) -> Result<()> {
    let database = model_handle.database();
    let data_object = if let Some(data_object) = database.data_object("neuron_explainer").await? {
        data_object
    } else {
        database
            .add_data_object("neuron_explainer", DataType::NeuronExplainer)
            .await?
    };

    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    let start = Instant::now();
    let data = fetch(num_layers, layer_size, url).await?;
    let fetch_time = start.elapsed();
    println!("Fetched data in {:?}", fetch_time);
    let start = Instant::now();
    for (explanation, NeuronIndex { layer, neuron }) in data
        .into_iter()
        .zip(NeuronIndex::iter(num_layers, layer_size))
    {
        model_handle
            .add_neuron_data(&data_object, layer, neuron, explanation.to_binary()?)
            .await?;
    }
    let add_time = start.elapsed();
    println!("Added data in {:?}", add_time);

    Ok(())
}

pub async fn retrieve_neuron_explainer_small(model_handle: &mut ModelHandle) -> Result<()> {
    println!("Retrieving neuron explainer data for GPT-2 small.");
    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    ensure!(num_layers == SMALL_NUM_LAYERS, "Model has wrong number of layers. GPT-2 small has {SMALL_NUM_LAYERS} but model has {num_layers}.");
    ensure!(layer_size == SMALL_LAYER_SIZE, "Model has wrong layer size. GPT-2 small has {SMALL_LAYER_SIZE} neurons per layer, but model has {layer_size}");
    fetch_to_database(model_handle, small_url).await
}

pub async fn retrieve_neuron_explainer_xl(model_handle: &mut ModelHandle) -> Result<()> {
    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    ensure!(num_layers == XL_NUM_LAYERS, "Model has wrong number of layers. GPT-2 XL has {XL_NUM_LAYERS} but model has {num_layers}.");
    ensure!(layer_size == XL_LAYER_SIZE, "Model has wrong layer size. GPT-2 XL has {XL_LAYER_SIZE} neurons per layer, but model has {layer_size}");
    fetch_to_database(model_handle, xl_url).await
}
