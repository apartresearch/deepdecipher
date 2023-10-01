use std::{panic, sync::Arc, time::Duration};

use crate::{
    data::{
        data_objects::{
            DataObject, Metadata, NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage,
        },
        data_types::DataType,
        DataTypeHandle, ModelHandle, NeuronIndex,
    },
    util::Progress,
    Index,
};

use anyhow::{bail, Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::{sync::Semaphore, task::JoinSet};

const NEUROSCOPE_BASE_URL: &str = "https://neuroscope.io/";
const RETRY_LIMIT: u32 = 5;

fn neuron_page_url(model: &str, neuron_index: NeuronIndex) -> String {
    let NeuronIndex {
        layer: layer_index,
        neuron: neuron_index,
    } = neuron_index;
    format!("{NEUROSCOPE_BASE_URL}{model}/{layer_index}/{neuron_index}.html")
}

pub async fn scrape_neuron_page<S: AsRef<str>>(
    model: S,
    neuron_index: NeuronIndex,
) -> Result<NeuroscopeNeuronPage> {
    let url = neuron_page_url(model.as_ref(), neuron_index);
    let client = Client::new();
    let res = client.get(&url).send().await?;
    let page = res.text().await?;
    let page = NeuroscopeNeuronPage::from_html_str(&page, neuron_index)?;
    Ok(page)
}

async fn scrape_neuron_page_to_database(
    model: &mut ModelHandle,
    data_type: &DataTypeHandle,
    neuron_index: NeuronIndex,
) -> Result<f32> {
    let page = if let Some(page_data) = model
        .neuron_data(data_type, neuron_index.layer, neuron_index.neuron)
        .await?
    {
        NeuroscopeNeuronPage::from_binary(page_data)?
    } else {
        let page = scrape_neuron_page(model.name(), neuron_index).await?;
        model.add_neuron_data( data_type, neuron_index.layer, neuron_index.neuron, page.to_binary()?).await.with_context(|| format!("Failed to write neuroscope page for neuron {neuron_index} in model '{model_name}' to database.", model_name = model.name()))?;
        page
    };
    let model_name = model.name();
    let first_text = page
        .texts()
        .get(0)
        .with_context(|| format!("Failed to get first text from neuroscope page for neuron {neuron_index} in model '{model_name}'."))?;
    let activation_range = first_text.max_activation() - first_text.min_activation();

    Ok(activation_range)
}

async fn scrape_layer_to_database(
    model: &mut ModelHandle,
    data_type: &DataTypeHandle,
    layer_index: u32,
    num_neurons: u32,
    progress: &mut Progress,
) -> Result<NeuroscopeLayerPage> {
    let page = if let Some(page_data) = model.layer_data(data_type, layer_index).await? {
        let layer_page = NeuroscopeLayerPage::from_binary(page_data)?;
        progress
            .increment_by(num_neurons.into())
            .expect("Should be impossible to exceed progress total.");
        layer_page
    } else {
        let mut join_set = JoinSet::new();

        let semaphore = Arc::new(Semaphore::new(20));

        for neuron_index in 0..num_neurons {
            let neuron_index = NeuronIndex {
                layer: layer_index,
                neuron: neuron_index,
            };

            let mut model = model.clone();
            let data_type = data_type.clone();

            let semaphore = Arc::clone(&semaphore);
            join_set.spawn(async move {
                let permit = semaphore.acquire_owned().await.unwrap();
                let mut retries = 0;
                    let result = loop {
                        match scrape_neuron_page_to_database(&mut model, &data_type, neuron_index).await {
                            Ok(result) => break result,
                            Err(err) => {
                                if retries == RETRY_LIMIT {
                                    log::error!("Failed to fetch neuroscope page for neuron {neuron_index} after {retries} retries. Error: {err:?}");
                                    return Err(err);
                                }
                                log::error!(
                                    "Failed to fetch neuroscope page for neuron {neuron_index}. Retrying...",
                                );
                                log::error!("Error: {err:?}");
                                retries += 1;
                                tokio::time::sleep(Duration::from_millis(5)).await;
                            }
                        }
                    };
                drop(permit);
                Ok::<_, anyhow::Error>((neuron_index, result))
            });
        }

        let mut max_activations = Vec::with_capacity(num_neurons as usize);

        while let Some(join_result) = join_set.join_next().await {
            let neuron_max_activation = match join_result {
                Ok(scrape_result) => scrape_result?,
                Err(join_error) => {
                    let panic_object = join_error
                        .try_into_panic()
                        .expect("Should be impossible to cancel these tasks.");
                    panic::resume_unwind(panic_object);
                }
            };
            max_activations.push(neuron_max_activation);
            progress.increment();
            progress.print();
        }

        let layer_page = NeuroscopeLayerPage::new(max_activations);
        model
            .add_layer_data(data_type, layer_index, layer_page.to_binary()?)
            .await?;
        layer_page
    };

    Ok(page)
}

pub async fn scrape_model_metadata<S: AsRef<str>>(model: S) -> Result<Metadata> {
    let model = model.as_ref();
    let url = NEUROSCOPE_BASE_URL;
    let client = Client::new();
    let response = client.get(url).send().await?;
    let page = response.text().await?;
    let document = Html::parse_document(&page);
    let model_name_selector = Selector::parse("td:nth-child(1) a").unwrap();
    let model_index = document
        .select(&model_name_selector)
        .map(|element| {
            element
                .text()
                .next()
                .expect("Model name should be a non-empty string.")
        })
        .position(|name| name == model)
        .with_context(|| format!("Neuroscope has no model with name {model}."))?;
    let row_selector = Selector::parse("tr").unwrap();
    let model_row = document.select(&row_selector).nth(model_index + 1).unwrap();
    let row_element_selector = Selector::parse("td").unwrap();
    let row_elements: Vec<_> = model_row
        .select(&row_element_selector)
        .map(|element| {
            element
                .text()
                .next()
                .expect("Model row element should be a non-empty string.")
        })
        .collect();
    let activation_function = row_elements[2].to_owned();
    let dataset = row_elements[3].to_owned();
    let num_layers = row_elements[4].replace(',', "").parse::<u32>().unwrap();
    let layer_size = row_elements[5].replace(',', "").parse::<u32>().unwrap();
    let num_total_neurons = row_elements[6].replace(',', "").parse::<u32>().unwrap();
    let num_total_parameters = row_elements[7].replace(',', "").parse::<u32>().unwrap();

    Ok::<_, anyhow::Error>(Metadata {
        name: model.to_owned(),
        num_layers,
        layer_size,
        activation_function,
        num_total_neurons,
        num_total_parameters,
        dataset,
    })
}

pub async fn scrape_model_to_database(model: &mut ModelHandle) -> Result<()> {
    let database = model.database();
    let data_type = if let Some(data_type) = database.data_type("neuroscope").await? {
        data_type
    } else {
        database
            .add_data_type("neuroscope", DataType::Neuroscope)
            .await?
    };
    if model.model_data(&data_type).await?.is_some() {
        println!(
            "Neuroscope pages for model '{}' already scraped.",
            model.name()
        );
        anyhow::Ok(())
    } else {
        let mut progress = Progress::start(
            (model.metadata().num_layers * model.metadata().layer_size) as u64,
            "Scraping neuroscope model",
        );
        let mut layer_pages = Vec::with_capacity(model.metadata().num_layers as usize);
        let layer_size = model.metadata().layer_size;
        for layer_index in 0..model.metadata().num_layers {
            let layer_page = scrape_layer_to_database(
                &mut model.clone(),
                &data_type,
                layer_index,
                layer_size,
                &mut progress,
            )
            .await?;
            layer_pages.push(layer_page)
        }

        println!("Scraped neuroscope pages for model '{}'.", model.name());

        let neuron_importance: Vec<(NeuronIndex, f32)> = layer_pages
            .into_iter()
            .flat_map(|layer_page| layer_page.important_neurons().to_vec())
            .collect();
        let model_page = NeuroscopeModelPage::new(neuron_importance);
        model
            .add_model_data(&data_type, model_page.to_binary()?)
            .await?;
        model.add_data_type(&data_type).await
    }
}

pub async fn scrape_indices_to_database(
    model: &mut ModelHandle,
    data_type: &DataTypeHandle,
    indices: impl Iterator<Item = Index>,
) -> Result<()> {
    match data_type.data_type() {
        DataType::Neuroscope => {}
        _ => bail!("Cannot scrape missing indices for non-neuroscope data object."),
    }

    let indices = indices.collect::<Vec<_>>();

    let mut progress = Progress::start(indices.len() as u64, "Scraping missing neuroscope items");
    for index in indices {
        match index {
            Index::Model => {
                bail!("Cannot handle model index.")
            }
            Index::Layer(_) => {
                bail!("Cannot handle layer index.")
            }
            Index::Neuron(layer_index, neuron_index) => {
                scrape_neuron_page_to_database(
                    model,
                    data_type,
                    NeuronIndex {
                        layer: layer_index,
                        neuron: neuron_index,
                    },
                )
                .await?;
                progress.increment();
                progress.print();
            }
        }
    }
    Ok(())
}

pub async fn scrape_missing_indices(
    model: &mut ModelHandle,
    data_type: &DataTypeHandle,
) -> Result<()> {
    match data_type.data_type() {
        DataType::Neuroscope => {}
        _ => bail!("Cannot scrape missing indices for non-neuroscope data object."),
    }
    let missing_indices = model.missing_items(data_type).await?;
    scrape_indices_to_database(model, data_type, missing_indices).await
}
