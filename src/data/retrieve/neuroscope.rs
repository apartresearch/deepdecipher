use std::{
    fs::{self, File},
    io::{self, Write},
    panic,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::data::{LayerMetadata, ModelMetadata, NeuroscopePage};

use anyhow::{Context, Result};
use itertools::Itertools;
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::{sync::Semaphore, task::JoinSet};

const NEUROSCOPE_BASE_URL: &str = "https://neuroscope.io/";

pub fn neuron_data_path<S: AsRef<str>, P: AsRef<Path>>(
    data_path: P,
    model: S,
    layer_index: u32,
    neuron_index: u32,
) -> PathBuf {
    data_path
        .as_ref()
        .join(model.as_ref())
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}"))
        .with_extension("postcard")
}

pub fn neuron_page_url(model: &str, layer_index: u32, neuron_index: u32) -> String {
    format!("{NEUROSCOPE_BASE_URL}{model}/{layer_index}/{neuron_index}.html")
}

pub async fn scrape_neuron_page<S: AsRef<str>>(
    model: S,
    layer_index: u32,
    neuron_index: u32,
) -> Result<NeuroscopePage> {
    let url = neuron_page_url(model.as_ref(), layer_index, neuron_index);
    let client = Client::new();
    let res = client.get(&url).send().await?;
    let page = res.text().await?;
    let page = NeuroscopePage::from_html_str(&page, layer_index, neuron_index)?;
    Ok(page)
}

pub async fn scrape_neuron_page_to_file<S: AsRef<str>, P: AsRef<Path>>(
    data_path: P,
    model: S,
    layer_index: u32,
    neuron_index: u32,
) -> Result<()> {
    let page_path = neuron_data_path(data_path, model.as_ref(), layer_index, neuron_index);
    if page_path.exists() {
        Ok(())
    } else {
        let page = scrape_neuron_page(model, layer_index, neuron_index).await?;
        page.to_file(page_path)
    }
}

pub async fn scrape_layer(
    model: &str,
    layer_index: u32,
    num_neurons: u32,
) -> Result<Vec<NeuroscopePage>> {
    let mut join_set = JoinSet::new();

    for neuron_index in 0..num_neurons {
        let model = model.to_owned();
        join_set.spawn(async move {
            (
                neuron_index,
                scrape_neuron_page(model, layer_index, neuron_index).await,
            )
        });
    }

    let mut pages = Vec::with_capacity(
        num_neurons
            .try_into()
            .expect("Are you running this on a potato? Apparently it's a 16-bit system or less?"),
    );
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok((neuron_index, page)) => {
                pages.push((neuron_index, page.with_context(|| format!("Failed to scrape page for neuron {neuron_index} in layer {layer_index} of model '{model}'."))?));
            }
            Err(join_error) => {
                let panic_object = join_error
                    .try_into_panic()
                    .expect("Should be impossible to cancel these tasks.");
                panic::resume_unwind(panic_object);
            }
        }
    }

    pages.sort_unstable_by_key(|(neuron_index, _)| *neuron_index);
    assert!(pages
        .iter()
        .tuple_windows()
        .all(|((neuron_index, _), (next_neuron_index, _))| {
            *neuron_index + 1 == *next_neuron_index
        }));

    let pages = pages.into_iter().map(|(_, page)| page).collect();

    Ok(pages)
}

pub async fn scrape_layer_to_files<P: AsRef<Path>, S: AsRef<str>>(
    data_path: P,
    model: S,
    layer_index: u32,
    num_neurons: u32,
) -> Result<()> {
    let mut join_set = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(20));

    for neuron_index in 0..num_neurons {
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

        let model = model.as_ref().to_owned();
        let data_path = data_path.as_ref().to_owned();
        join_set.spawn(async move {
            let result =
                scrape_neuron_page_to_file(data_path, model, layer_index, neuron_index).await;
            drop(permit);
            result
        });
    }

    println!("Scraping pages...");
    print!("Pages scraped: 0/{num_neurons}",);
    io::stdout().flush().unwrap();
    let mut num_completed = 0;
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(scrape_result) => scrape_result?,
            Err(join_error) => {
                let panic_object = join_error
                    .try_into_panic()
                    .expect("Should be impossible to cancel these tasks.");
                panic::resume_unwind(panic_object);
            }
        }
        num_completed += 1;
        print!("\rPages scraped: {num_completed}/{num_neurons}");
        io::stdout().flush().unwrap();
    }
    assert_eq!(
        num_completed, num_neurons,
        "Should have scraped all pages. Only scaped {num_completed}/{num_neurons} pages."
    );
    println!("\rPages scraped: {num_neurons}/{num_neurons}");
    io::stdout().flush().unwrap();

    Ok(())
}

pub async fn scrape_model_metadata<S: AsRef<str>>(model: S) -> Result<ModelMetadata> {
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
    let num_neurons_per_layer = row_elements[5].replace(',', "").parse::<u32>().unwrap();
    let layers: Vec<_> = (0..num_layers)
        .map(|_| LayerMetadata {
            num_neurons: num_neurons_per_layer,
        })
        .collect();
    let num_total_neurons = row_elements[6].replace(',', "").parse::<u32>().unwrap();
    let num_total_parameters = row_elements[7].replace(',', "").parse::<u32>().unwrap();

    Ok::<_, anyhow::Error>(ModelMetadata {
        name: model.to_owned(),
        layers,
        activation_function,
        num_total_neurons,
        num_total_parameters,
        dataset,
    })
}

pub async fn scrape_model_metadata_to_file<P: AsRef<Path>, S: AsRef<str>>(
    data_path: P,
    model: S,
) -> Result<()> {
    let model = model.as_ref();
    let model_metadata = scrape_model_metadata(model).await?;
    let model_metadata_path = data_path.as_ref().join(model).join("metadata.json");
    fs::create_dir_all(
        model_metadata_path
            .parent()
            .with_context(|| format!("Invalid path '{model_metadata_path:?}'"))?,
    )
    .with_context(|| format!("Failed to create directory for '{model_metadata_path:?}'"))?;
    let model_metadata_file = File::create(model_metadata_path)?;

    serde_json::to_writer(model_metadata_file, &model_metadata)?;
    Ok(())
}
