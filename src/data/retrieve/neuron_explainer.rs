use std::{
    panic,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{bail, ensure, Result};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
    data::{
        data_objects::{DataObject, NeuronExplainerPage},
        data_types::DataType,
        DataTypeHandle, ModelHandle, NeuronIndex,
    },
    util::Progress,
};

const SMALL_NUM_LAYERS: u32 = 12;
const SMALL_LAYER_SIZE: u32 = 3072;
const XL_NUM_LAYERS: u32 = 48;
const XL_LAYER_SIZE: u32 = 6400;

const RETRY_LIMIT: u32 = 5;

fn small_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/gpt2_small_data/explanations/{layer}/{neuron}.jsonl")
}

fn xl_url(index: NeuronIndex) -> String {
    let NeuronIndex { layer, neuron } = index;
    format!("https://openaipublic.blob.core.windows.net/neuron-explainer/data/explanations/{layer}/{neuron}.jsonl")
}

pub fn model_url(model_name: &str, index: NeuronIndex) -> Result<String> {
    match model_name {
        "gpt2-small" => Ok(small_url(index)),
        "gpt2-xl" => Ok(xl_url(index)),
        _ => bail!(
            "Neuron explainer retrieval only available for models 'gpt2-small' and 'gpt2-xl'. \
             Given model name: {model_name}"
        ),
    }
}

pub async fn fetch_neuron(client: &Client, url: impl AsRef<str>) -> Result<NeuronExplainerPage> {
    let res = client.get(url.as_ref()).send().await?;
    let page = res.json::<serde_json::Value>().await?;

    NeuronExplainerPage::from_json(page)
}

async fn fetch(
    model_handle: &mut ModelHandle,
    data_type: &DataTypeHandle,
    num_layers: u32,
    layer_size: u32,
    url: impl Fn(NeuronIndex) -> String,
) -> Result<()> {
    let mut join_set = JoinSet::new();
    let client = Client::new();

    let semaphore = Arc::new(Semaphore::new(20));

    let mut progress = Progress::start((num_layers * layer_size) as u64, "Fetching data");
    progress.print();
    for index in NeuronIndex::iter(num_layers, layer_size) {
        if model_handle
            .neuron_data(data_type, index.layer, index.neuron)
            .await?
            .is_none()
        {
            let url = url(index);

            let semaphore = Arc::clone(&semaphore);
            let client = client.clone();
            join_set.spawn(async move {
                let permit = semaphore.acquire_owned().await.unwrap();

                let mut retries = 0;
                let result = loop {
                    match fetch_neuron(&client, &url).await {
                        Ok(result) => break Some(result),
                        Err(err) => {
                            if retries == RETRY_LIMIT {
                                log::error!(
                                    "Failed to fetch neuron explainer data for neuron {index} \
                                     after {retries} retries. Error: {err}"
                                );
                                break None;
                            }
                            log::error!(
                                "Failed to fetch neuron explainer data for neuron {index}. \
                                 Retrying...",
                            );
                            log::error!("Error: {err}");
                            retries += 1;
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    }
                };
                drop(permit);
                (index, result)
            });
        } else {
            progress.increment();
        }
    }

    while let Some(join_result) = join_set.join_next().await {
        let (NeuronIndex { layer, neuron }, page) = match join_result {
            Ok(scrape_result) => scrape_result,
            Err(join_error) => {
                let panic_object = join_error
                    .try_into_panic()
                    .expect("Should be impossible to cancel these tasks.");
                panic::resume_unwind(panic_object);
            }
        };
        if let Some(explanation) = page {
            model_handle
                .add_neuron_data(data_type, layer, neuron, explanation.to_binary()?)
                .await?;
        }
        progress.increment();
        progress.print();
    }

    println!("Data fetched.                                          ");
    Ok(())
}

async fn fetch_to_database(
    model_handle: &mut ModelHandle,
    url: impl Fn(NeuronIndex) -> String,
) -> Result<()> {
    let database = model_handle.database();
    let data_type = if let Some(data_type) = database.data_type("neuron_explainer").await? {
        data_type
    } else {
        database
            .add_data_type("neuron_explainer", DataType::NeuronExplainer)
            .await?
    };

    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    let start = Instant::now();
    fetch(model_handle, &data_type, num_layers, layer_size, url).await?;
    let fetch_time = start.elapsed();
    println!("Fetched data in {:?}", fetch_time);

    if !model_handle.has_data_type(&data_type).await? {
        model_handle.add_data_type(&data_type).await?;
    }

    Ok(())
}

pub async fn retrieve_neuron_explainer_small(model_handle: &mut ModelHandle) -> Result<()> {
    println!("Retrieving neuron explainer data for GPT-2 small.");
    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    ensure!(
        num_layers == SMALL_NUM_LAYERS,
        "Model has wrong number of layers. GPT-2 small has {SMALL_NUM_LAYERS} but model has \
         {num_layers}."
    );
    ensure!(
        layer_size == SMALL_LAYER_SIZE,
        "Model has wrong layer size. GPT-2 small has {SMALL_LAYER_SIZE} neurons per layer, but \
         model has {layer_size}"
    );
    fetch_to_database(model_handle, small_url).await
}

pub async fn retrieve_neuron_explainer_xl(model_handle: &mut ModelHandle) -> Result<()> {
    let num_layers = model_handle.metadata().num_layers;
    let layer_size = model_handle.metadata().layer_size;
    ensure!(
        num_layers == XL_NUM_LAYERS,
        "Model has wrong number of layers. GPT-2 XL has {XL_NUM_LAYERS} but model has \
         {num_layers}."
    );
    ensure!(
        layer_size == XL_LAYER_SIZE,
        "Model has wrong layer size. GPT-2 XL has {XL_LAYER_SIZE} neurons per layer, but model \
         has {layer_size}"
    );
    fetch_to_database(model_handle, xl_url).await
}
