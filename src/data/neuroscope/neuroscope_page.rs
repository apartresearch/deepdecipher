use std::str::FromStr;

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use snap::raw::{Decoder, Encoder};

use crate::data::NeuronIndex;

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
pub struct NeuroscopeNeuronPage {
    neuron_index: NeuronIndex,
    texts: Vec<Text>,
}

impl NeuroscopeNeuronPage {
    fn from_html_header_and_texts(
        header_html: &str,
        texts: Vec<Text>,
        neuron_index: NeuronIndex,
    ) -> Result<Self> {
        let neuron_index_regex = Regex::new(r"<h2>Neuron (\d+) in Layer (\d+) </h2>")
            .context("Failed to create regex.")?;

        let captures = neuron_index_regex
            .captures_iter(header_html)
            .at_most_one()
            .expect("Multiple neuron index headers found.") // TODO: Figure out how to anyhow this error.
            .context("Failed to find neuron index header.")?;
        let html_layer_index = captures[2]
            .parse::<u32>()
            .context("Failed to parse layer index.")?;
        let html_neuron_index = captures[1]
            .parse::<u32>()
            .context("Failed to parse neuron index.")?;
        let html_neuron_index = NeuronIndex {
            layer: html_layer_index,
            neuron: html_neuron_index,
        };

        assert_eq!(neuron_index, html_neuron_index);

        Ok(Self {
            neuron_index,
            texts,
        })
    }

    pub fn from_html_str(html: &str, neuron_index: NeuronIndex) -> Result<Self> {
        let mut sections = html.split("<hr>");
        let header = sections.next().context("Tag &lt;hr&gt; not found.")?;
        let nothing = sections
            .next()
            .context("Second &lt;hr&gt; tag not found.")?;
        if !nothing.trim().is_empty() {
            bail!("Space between first two &lt;hr&gt; tags not empty.");
        }

        let texts = sections
            .enumerate()
            .map(|(index, html)| {
                Text::from_html_str(html).with_context(|| format!("Failed to parse text {index}."))
            })
            .collect::<Result<Vec<Text>>>()?;

        Self::from_html_header_and_texts(header, texts, neuron_index)
    }

    pub fn to_binary(&self) -> Result<Vec<u8>> {
        let data =
            postcard::to_allocvec(self).context("Failed to serialize neuroscope neuron page.")?;
        Encoder::new()
            .compress_vec(data.as_slice())
            .context("Failed to compress neuroscope neuron page.")
    }

    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        let data = Decoder::new()
            .decompress_vec(data.as_ref())
            .context("Failed to decompress neuroscope neuron page")?;
        postcard::from_bytes(data.as_slice())
            .context("Failed to deserialize neuroscope neuron page.")
    }

    pub fn neuron_index(&self) -> NeuronIndex {
        self.neuron_index
    }

    pub fn texts(&self) -> &[Text] {
        self.texts.as_slice()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    min_range: f32,
    max_range: f32,
    min_activation: f32,
    max_activation: f32,
    data_index: u64,
    max_activating_token_index: u32,
    tokens: Vec<String>,
    activations: Vec<f32>,
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

        let text_regex = Regex::new(r"ColoredTokens,\s*(.*\})\s*\)\s*</script>\s*</details>")
            .context("Failed to create regex.")?;
        let text: String = regex(&text_regex, html, "Text")?;
        let text_json = serde_json::from_str::<serde_json::Value>(&text)
            .context("Failed to parse text JSON.")?;
        let tokens = text_json
            .get("tokens")
            .context("Failed to get tokens from text JSON.")?
            .as_array()
            .context("Tokens JSON is not an array.")?
            .iter()
            .map(|token_json| token_json.as_str().context("Token not a string"))
            .collect::<Result<Vec<_>>>()?;
        let tokens = tokens.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let activations = text_json
            .get("values")
            .context("Failed to get activations from text JSON.")?
            .as_array()
            .context("Activations JSON is not an array.")?
            .iter()
            .map(|activation_json| {
                activation_json
                    .as_f64()
                    .context("Activation not a float")
                    .map(|activation| activation as f32)
            })
            .collect::<Result<Vec<_>>>()?;

        if tokens.len() != activations.len() {
            bail!("Tokens and activations have different lengths.")
        }

        Ok(Self {
            max_activating_token_index,
            max_range,
            min_range,
            max_activation: max_act,
            min_activation: min_act,
            data_index,
            tokens,
            activations,
        })
    }

    pub fn min_activation(&self) -> f32 {
        self.min_activation
    }

    pub fn max_activation(&self) -> f32 {
        self.max_activation
    }

    pub fn tokens(&self) -> &[String] {
        self.tokens.as_slice()
    }

    pub fn activations(&self) -> &[f32] {
        self.activations.as_slice()
    }
}
