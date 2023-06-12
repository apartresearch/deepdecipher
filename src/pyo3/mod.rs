use crate::{
    data::{retrieve, NeuronViewerObject, TokenDictionary},
    server,
};
use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyfunction]
fn debug() {
    println!("Hello from Rust!");
}

#[pyfunction]
fn start_server() {
    server::start_server().unwrap();
}

#[pyfunction]
fn scrape_layer_to_files(
    data_path: &str,
    model: &str,
    tokens: Vec<String>,
    layer_index: u32,
    num_neurons: u32,
) -> PyResult<()> {
    Runtime::new()
        .context("Failed to start async runtime to scrape neuroscope.")?
        .block_on(async {
            println!("Scraping layer {layer_index} of model {model} to {data_path}...");
            let token_dictionary = TokenDictionary::new(tokens);
            retrieve::neuroscope::scrape_layer_to_files(
                data_path,
                model,
                token_dictionary,
                layer_index,
                num_neurons,
            )
            .await
            .context("Failed to scrape layer.")
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

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(debug, m)?)?;
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_layer_to_files, m)?)?;
    m.add_class::<PyNeuronViewerObject>()?;
    Ok(())
}
