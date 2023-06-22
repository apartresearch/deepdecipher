use crate::{
    data::{retrieve, NeuronViewerObject, NeuroscopePage},
    server,
};
use anyhow::{Context, Result};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyfunction]
fn start_server() {
    server::start_server().unwrap();
}

#[pyfunction]
fn scrape_layer_to_files(
    data_path: &str,
    model: &str,
    layer_index: u32,
    num_neurons: u32,
) -> PyResult<()> {
    Runtime::new()
        .context("Failed to start async runtime to scrape neuroscope.")?
        .block_on(async {
            println!("Scraping layer {layer_index} of model {model} to {data_path}.");
            retrieve::neuroscope::scrape_layer_to_files(data_path, model, layer_index, num_neurons)
                .await
                .context("Failed to scrape layer.")
        })?;
    Ok(())
}

#[pyfunction]
fn scrape_model_metadata_to_file(data_path: &str, model: &str) -> PyResult<()> {
    Runtime::new()
        .context("Failed to start async runtime to scrape neuroscope.")?
        .block_on(async {
            println!("Scraping metadata of model {model} to {data_path}.");
            retrieve::neuroscope::scrape_model_metadata_to_file(data_path, model)
                .await
                .context("Failed to scrape model metadata.")
        })?;
    Ok(())
}

#[pyclass(name = "NeuronViewerObject")]
struct PyNeuronViewerObject {
    object: NeuronViewerObject,
}

#[pymethods]
impl PyNeuronViewerObject {
    #[new]
    fn new(json: &str) -> PyResult<Self> {
        let json_value = serde_json::from_str(json).context("failed to parse json")?;
        Ok(PyNeuronViewerObject {
            object: NeuronViewerObject::from_json(&json_value)?,
        })
    }

    fn to_file(&self, path: &str) {
        self.object.to_file(path);
    }
}

#[pyclass(name = "NeuroscopePage")]
struct PyNeuroscopePage {
    object: NeuroscopePage,
}

#[pymethods]
impl PyNeuroscopePage {
    #[new]
    fn new(html: &str, layer_index: u32, neuron_index: u32) -> PyResult<Self> {
        Ok(PyNeuroscopePage {
            object: NeuroscopePage::from_html_str(html, layer_index, neuron_index)
                .with_context(|| format!("Failed to parse html of neuroscope page for neuron index {neuron_index} on layer {layer_index}."))?,
        })
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.object).context("Failed to serialize neuroscope page.")
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.object)
    }

    fn to_file(&self, path: &str) -> Result<()> {
        self.object
            .to_file(path)
            .with_context(|| format!("Failed to write neuroscope page to file '{path:?}'."))
    }

    #[staticmethod]
    fn from_file(path: &str) -> Result<Self> {
        Ok(PyNeuroscopePage {
            object: NeuroscopePage::from_file(path)
                .with_context(|| format!("Failed to read neuroscope page from file '{path:?}'."))?,
        })
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_layer_to_files, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_model_metadata_to_file, m)?)?;
    m.add_class::<PyNeuronViewerObject>()?;
    m.add_class::<PyNeuroscopePage>()?;
    Ok(())
}
