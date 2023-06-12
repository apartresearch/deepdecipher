use std::{
    io::{self, Write},
    panic,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::data::{NeuroscopePage, TokenDictionary};

use anyhow::{Context, Result};
use reqwest::Client;
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
    token_dictionary: &TokenDictionary,
    layer_index: u32,
    neuron_index: u32,
) -> Result<NeuroscopePage> {
    let url = neuron_page_url(model.as_ref(), layer_index, neuron_index);
    let client = Client::new();
    let res = client.get(&url).send().await?;
    let page = res.text().await?;
    let page = NeuroscopePage::from_html_str(&page, token_dictionary, layer_index, neuron_index)?;
    Ok(page)
}

pub async fn scrape_neuron_page_to_file<S: AsRef<str>, P: AsRef<Path>>(
    data_path: P,
    model: S,
    token_dictionary: &TokenDictionary,
    layer_index: u32,
    neuron_index: u32,
) -> Result<()> {
    let page_path = neuron_data_path(data_path, model.as_ref(), layer_index, neuron_index);
    if page_path.exists() {
        Ok(())
    } else {
        let page = scrape_neuron_page(model, token_dictionary, layer_index, neuron_index).await?;
        page.to_file(page_path)
    }
}

pub async fn scrape_layer_to_files<P: AsRef<Path>, S: AsRef<str>>(
    data_path: P,
    model: S,
    token_dictionary: TokenDictionary,
    layer_index: u32,
    num_neurons: u32,
) -> Result<()> {
    let mut join_set = JoinSet::new();

    let token_dictionary = Arc::new(token_dictionary);
    let semaphore = Arc::new(Semaphore::new(20));

    for neuron_index in 0..num_neurons {
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

        let model = model.as_ref().to_owned();
        let data_path = data_path.as_ref().to_owned();
        let token_dictionary = Arc::clone(&token_dictionary);

        join_set.spawn(async move {
            let result = scrape_neuron_page_to_file(
                data_path,
                model,
                &token_dictionary,
                layer_index,
                neuron_index,
            )
            .await;
            drop(permit);
            (neuron_index, result)
        });
    }

    println!("Scraping pages...");
    print!("Pages scraped: 0/{num_neurons}",);
    io::stdout().flush().unwrap();
    let mut num_completed = 0;
    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok((neuron_index, scrape_result)) => scrape_result.context(format!(
                "Failed to parse the page for neuron {neuron_index}."
            ))?,
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
