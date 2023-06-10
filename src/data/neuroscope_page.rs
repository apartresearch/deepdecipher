use std::str::FromStr;

use anyhow::{Context, Result};
use itertools::Itertools;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const FLOAT_REGEX: &str = r"-?\d+(?:\.\d*)?";

fn regex<T>(regex: &Regex, html: &str, search_name: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let found_string = &regex
        .captures_iter(html)
        .at_most_one()
        .unwrap_or_else(|_| panic!("Multiple \"{search_name}\"s found."))
        .with_context(|| format!("No \"{search_name}\" found."))?[1];
    found_string.parse::<T>().with_context(|| {
        format!(
            "Failed to parse \"{search_name}\" to type {}. Found: {found_string}",
            std::any::type_name::<T>()
        )
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroscopePage {
    neuron_index: u32,
    layer_index: u32,
    texts: Vec<Text>,
}

impl NeuroscopePage {
    fn from_html_header_and_texts(
        header_html: &str,
        texts: Vec<Text>,
        neuron_index: u32,
        layer_index: u32,
    ) -> Result<Self> {
        let neuron_index_regex = Regex::new(r"<h2>Neuron (\d+) in Layer (\d+) </h2>")
            .context("Failed to create regex.")?;

        let captures = neuron_index_regex
            .captures_iter(header_html)
            .at_most_one()
            .expect("Multiple neuron index headers found.") // TODO: Figure out how to anyhow this error.
            .context("Failed to find neuron index header.")?;
        let html_layer_index = captures[1]
            .parse::<u32>()
            .context("Failed to parse layer index.")?;
        let html_neuron_index = captures[2]
            .parse::<u32>()
            .context("Failed to parse neuron index.")?;

        assert_eq!(layer_index, html_layer_index);
        assert_eq!(neuron_index, html_neuron_index);

        Ok(Self {
            neuron_index,
            layer_index,
            texts,
        })
    }

    pub fn from_html_str(html: &str, neuron_index: u32, layer_index: u32) -> Result<Self> {
        let mut sections = html.split("<hr>");
        let header = sections.next().context("Tag <hr> not found.")?;
        let nothing = sections.next().context("Second <hr> tag not found.")?;
        assert_eq!(nothing, "", "Space between first two <hr> tags not empty.");

        let texts = sections
            .map(Text::from_html_str)
            .collect::<Result<Vec<Text>>>()?;

        Self::from_html_header_and_texts(header, texts, neuron_index, layer_index)
    }

    pub fn from_html(document: Html, neuron_index: u32, layer_index: u32) -> Result<Self> {
        let index_scraper = Selector::parse("h1+ h2").expect("Invalid selector.");
        let index_text = document
            .select(&index_scraper)
            .next()
            .context("Neuron index not found.")?
            .text()
            .next()
            .context("No text in neuron index header.")?;
        let values = index_text
            .strip_prefix("Neuron ")
            .context("Neuron index header doesn't start with 'Neuron '")?
            .split(" in Layer ")
            .map(|s| s.trim().parse::<u32>())
            .collect::<Result<Vec<u32>, _>>()
            .context("Failed to parse neuron index header.")?;
        if values.len() != 2 {
            return Err(anyhow::anyhow!(
                "Neuron index header doesn't contain exactly one ' in Layer '"
            ));
        }

        assert_eq!(values, vec![layer_index, neuron_index,]);

        let text_selector = Selector::parse(".colored-tokens").expect("Invalid selector.");
        let meta_data_selector = Selector::parse("h4").expect("Invalid selector.");

        let texts = document
            .select(&text_selector)
            .map(|text| text.text().next().unwrap().to_string())
            .collect::<Vec<String>>();
        println!("{texts:?}");

        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    max_range: f32,
    min_range: f32,
    max_act: f32,
    min_act: f32,
    data_index: u64,
    max_activating_token_index: u32,
}

impl Text {
    pub fn from_html_str(html: &str) -> Result<Self> {
        let max_range_regex = Regex::new(&format!(r"<h4>Max Range: <b>({FLOAT_REGEX})</b>."))
            .context("Failed to create regex.")?;
        let max_range = regex(&max_range_regex, html, "Max Range")?;

        let min_range_regex = Regex::new(&format!(r"</b>. Min Range: <b>({FLOAT_REGEX})</b></h4>"))
            .context("Failed to create regex.")?;
        let min_range = regex(&min_range_regex, html, "Min Range")?;

        let max_act_regex = Regex::new(&format!(r"<h4>Max Act: <b>({FLOAT_REGEX})</b>."))
            .context("Failed to create regex.")?;
        let max_act = regex(&max_act_regex, html, "Max Act")?;

        let min_act_regex = Regex::new(&format!(r"</b>. Min Act: <b>({FLOAT_REGEX})</b></h4>"))
            .context("Failed to create regex.")?;
        let min_act = regex(&min_act_regex, html, "Min Act")?;

        let data_index_regex =
            Regex::new(r"<h4>Data Index: <b>(\d+)</b>").context("Failed to create regex.")?;
        let data_index = regex(&data_index_regex, html, "Data Index")?;

        let max_activating_token_index_regex =
            Regex::new(r"<h4>Max Activating Token Index: <b>(\d+)</b></h4>")
                .context("Failed to create regex.")?;
        let max_activating_token_index = regex(
            &max_activating_token_index_regex,
            html,
            "Max Activating Token Index",
        )?;

        Ok(Self {
            max_activating_token_index,
            max_range,
            min_range,
            max_act,
            min_act,
            data_index,
        })
    }
}
