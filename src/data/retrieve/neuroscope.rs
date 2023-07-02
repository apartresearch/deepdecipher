use std::{
    io::{self, Write},
    panic,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::data::{
    data_types::DataType,
    database::Database,
    neuroscope::{NeuroscopeLayerPage, NeuroscopeModelPage},
    LayerMetadata, Metadata, NeuronIndex, NeuroscopeNeuronPage,
};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::{sync::Semaphore, task::JoinSet};

const NEUROSCOPE_BASE_URL: &str = "https://neuroscope.io/";

pub fn neuron_data_path<S: AsRef<str>, P: AsRef<Path>>(
    data_path: P,
    model: S,
    neuron_index: NeuronIndex,
) -> PathBuf {
    let NeuronIndex {
        layer: layer_index,
        neuron: neuron_index,
    } = neuron_index;
    data_path
        .as_ref()
        .join(model.as_ref())
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}"))
        .with_extension("postcard")
}

pub fn neuron_page_url(model: &str, neuron_index: NeuronIndex) -> String {
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

pub async fn scrape_neuron_page_to_file<S: AsRef<str>, P: AsRef<Path>>(
    data_path: P,
    model: S,
    neuron_index: NeuronIndex,
) -> Result<f32> {
    let model = model.as_ref();
    let page_path = neuron_data_path(data_path, model, neuron_index);
    let page = if page_path.exists() {
        NeuroscopeNeuronPage::from_file(page_path).with_context(|| format!("File for neuroscape page exists, but cannot be loaded. Neuron {neuron_index} in model '{model}'."))?
    } else {
        let page = scrape_neuron_page(model, neuron_index).await?;
        page.to_file(page_path).with_context(|| format!("Failed to write neuroscope page to file for neuron {neuron_index} in model '{model}'."))?;
        page
    };
    let first_text = page
        .texts()
        .get(0)
        .with_context(|| format!("Failed to get first text from neuroscope page for neuron {neuron_index} in model '{model}'."))?;
    let activation_range = first_text.max_activation() - first_text.min_activation();

    Ok(activation_range)
}

pub async fn scrape_neuron_page_to_database(
    database: &Database,
    model_name: impl AsRef<str>,
    neuron_index: NeuronIndex,
) -> Result<f32> {
    let model_name = model_name.as_ref();
    let model = database
        .model(model_name.to_owned())
        .await?
        .with_context(|| format!("No model '{model_name}' in database."))?;
    let page = if let Some(page_data) = model
        .get_neuron_data("neuroscope", neuron_index.layer, neuron_index.neuron)
        .await?
    {
        NeuroscopeNeuronPage::from_binary(page_data)?
    } else {
        let page = scrape_neuron_page(model.name(), neuron_index).await?;
        model.add_neuron_data( "neuroscope", neuron_index.layer, neuron_index.neuron, page.to_binary()?).await.with_context(|| format!("Failed to write neuroscope page for neuron {neuron_index} in layer {layer_index} of model '{model_name}' to database.", neuron_index = neuron_index.neuron, layer_index = neuron_index.layer))?;
        page
    };
    let first_text = page
        .texts()
        .get(0)
        .with_context(|| format!("Failed to get first text from neuroscope page for neuron {neuron_index} in model '{model_name}'."))?;
    let activation_range = first_text.max_activation() - first_text.min_activation();

    Ok(activation_range)
}

pub async fn scrape_layer(
    model: &str,
    layer_index: u32,
    num_neurons: u32,
) -> Result<Vec<NeuroscopeNeuronPage>> {
    let mut join_set = JoinSet::new();

    for neuron_index in 0..num_neurons {
        let neuron_index = NeuronIndex {
            neuron: neuron_index,
            layer: layer_index,
        };
        let model = model.to_owned();
        join_set
            .spawn(async move { (neuron_index, scrape_neuron_page(model, neuron_index).await) });
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
            neuron_index.neuron + 1 == next_neuron_index.neuron
        }));

    let pages = pages.into_iter().map(|(_, page)| page).collect();

    Ok(pages)
}

pub async fn scrape_layer_to_files<P: AsRef<Path>, S: AsRef<str>>(
    data_path: P,
    model: S,
    layer_index: u32,
    num_neurons: u32,
) -> Result<NeuroscopeLayerPage> {
    let data_path = data_path.as_ref();

    let mut join_set = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(20));

    println!("Scraping pages...");
    print!("Pages scraped: 0/{num_neurons}",);

    for neuron_index in 0..num_neurons {
        let neuron_index = NeuronIndex {
            layer: layer_index,
            neuron: neuron_index,
        };
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

        let model = model.as_ref().to_owned();
        let data_path = data_path.to_owned();
        join_set.spawn(async move {
            let result = scrape_neuron_page_to_file(data_path, model, neuron_index).await;
            drop(permit);
            Ok::<_, anyhow::Error>((neuron_index, result?))
        });
    }

    let mut max_activations = Vec::with_capacity(num_neurons as usize);

    io::stdout().flush().unwrap();
    let mut num_completed = 0;
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
        num_completed += 1;
        print!("\rPages scraped: {num_completed}/{num_neurons}");
        io::stdout().flush().unwrap();
    }

    let layer_page = NeuroscopeLayerPage::new(max_activations);
    let layer_page_path = data_path
        .join(model.as_ref())
        .join("neuroscope")
        .join(format!("l{layer_index}"))
        .with_extension("postcard");
    layer_page.to_file(layer_page_path)?;

    assert_eq!(
        num_completed, num_neurons,
        "Should have scraped all pages. Only scaped {num_completed}/{num_neurons} pages."
    );
    println!("\rPages scraped: {num_neurons}/{num_neurons}");
    io::stdout().flush().unwrap();

    Ok(layer_page)
}

pub async fn scrape_layer_to_database(
    database: &Database,
    model_name: &str,
    layer_index: u32,
    num_neurons: u32,
) -> Result<NeuroscopeLayerPage> {
    let mut join_set = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(20));

    println!("Scraping pages...");
    print!("Pages scraped: 0/{num_neurons}",);

    for neuron_index in 0..num_neurons {
        let neuron_index = NeuronIndex {
            layer: layer_index,
            neuron: neuron_index,
        };

        let model_name = model_name.to_owned();
        let database = database.clone();
        let semaphore = Arc::clone(&semaphore);
        join_set.spawn(async move {
            let permit = semaphore.acquire_owned().await.unwrap();
            let result = scrape_neuron_page_to_database(&database, model_name, neuron_index).await;
            drop(permit);
            Ok::<_, anyhow::Error>((neuron_index, result?))
        });
    }

    let mut max_activations = Vec::with_capacity(num_neurons as usize);

    io::stdout().flush().unwrap();
    let mut num_completed = 0;
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
        num_completed += 1;
        print!("\rPages scraped: {num_completed}/{num_neurons}");
        io::stdout().flush().unwrap();
    }

    let layer_page = NeuroscopeLayerPage::new(max_activations);

    let model = database
        .model(model_name.to_owned())
        .await?
        .with_context(|| format!("No model '{model_name}' in database."))?;
    model
        .add_layer_data("neuroscope", layer_index, layer_page.to_binary()?)
        .await?;

    assert_eq!(
        num_completed, num_neurons,
        "Should have scraped all pages. Only scraped {num_completed}/{num_neurons} pages."
    );
    println!("\rPages scraped: {num_neurons}/{num_neurons}");
    io::stdout().flush().unwrap();

    Ok(layer_page)
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
    let num_neurons_per_layer = row_elements[5].replace(',', "").parse::<u32>().unwrap();
    let layers: Vec<_> = (0..num_layers)
        .map(|_| LayerMetadata {
            num_neurons: num_neurons_per_layer,
        })
        .collect();
    let num_total_neurons = row_elements[6].replace(',', "").parse::<u32>().unwrap();
    let num_total_parameters = row_elements[7].replace(',', "").parse::<u32>().unwrap();

    Ok::<_, anyhow::Error>(Metadata {
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
    model_metadata.to_file(data_path)
}

pub async fn scrape_model_to_files<P: AsRef<Path>, S: AsRef<str>>(
    data_path: P,
    model: S,
) -> Result<()> {
    let model = model.as_ref();
    let data_path = data_path.as_ref();

    let model_metadata = scrape_model_metadata(model).await?;
    model_metadata.to_file(data_path)?;

    let mut layer_pages = Vec::with_capacity(model_metadata.layers.len());
    for (layer_index, LayerMetadata { num_neurons }) in model_metadata.layers.iter().enumerate() {
        let layer_page =
            scrape_layer_to_files(data_path, model, layer_index as u32, *num_neurons).await?;
        layer_pages.push(layer_page)
    }
    let neuron_importance: Vec<(NeuronIndex, f32)> = layer_pages
        .into_iter()
        .flat_map(|layer_page| layer_page.important_neurons().to_vec())
        .collect();
    let model_page = NeuroscopeModelPage::new(neuron_importance);
    model_page.to_file(
        data_path
            .join(model)
            .join("neuroscope")
            .join("model")
            .with_extension("postcard"),
    )?;

    Ok(())
}

pub async fn scrape_model_to_database(
    database: &Database,
    model_name: impl AsRef<str>,
) -> Result<()> {
    let model_name = model_name.as_ref();

    let model = if let Some(model) = database.model(model_name.to_owned()).await? {
        model
    } else {
        let model_metadata = scrape_model_metadata(model_name).await?;
        database.add_model(model_metadata).await?
    };

    if database.data_object_type("neuroscope").await?.is_none() {
        database
            .add_data_object("neuroscope", DataType::Neuroscope)
            .await?;
    }

    if model.has_data_object("neuroscope").await? {
        bail!("Model '{model_name}' already has neuroscope data in database.")
    } else {
        model.add_data_object("neuroscope").await?
    }

    let mut layer_pages = Vec::with_capacity(model.metadata().layers.len());
    for (layer_index, LayerMetadata { num_neurons }) in model.metadata().layers.iter().enumerate() {
        let layer_page =
            scrape_layer_to_database(database, model.name(), layer_index as u32, *num_neurons)
                .await?;
        layer_pages.push(layer_page)
    }

    let neuron_importance: Vec<(NeuronIndex, f32)> = layer_pages
        .into_iter()
        .flat_map(|layer_page| layer_page.important_neurons().to_vec())
        .collect();
    let model_page = NeuroscopeModelPage::new(neuron_importance);
    model
        .add_model_data("neuroscope", model_page.to_binary()?)
        .await
}
