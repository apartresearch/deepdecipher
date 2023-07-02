use crate::{
    data::{database::Database, retrieve, NeuronIndex, NeuronViewerObject, NeuroscopeNeuronPage},
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
            println!("Scraping layer {layer_index} of model '{model}' to '{data_path}'.");
            retrieve::neuroscope::scrape_layer_to_files(data_path, model, layer_index, num_neurons)
                .await
                .context("Failed to scrape layer.")
        })?;
    Ok(())
}

#[pyfunction]
fn scrape_model_to_files(data_path: &str, model: &str) -> PyResult<()> {
    Runtime::new()
        .context("Failed to start async runtime to scrape neuroscope.")?
        .block_on(async {
            println!("Scraping model '{model}' to '{data_path}'.");
            retrieve::neuroscope::scrape_model_to_files(data_path, model)
                .await
                .context("Failed to scrape model.")
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
    object: NeuroscopeNeuronPage,
}

#[pymethods]
impl PyNeuroscopePage {
    #[new]
    fn new(html: &str, layer_index: u32, neuron_index: u32) -> PyResult<Self> {
        let neuron_index = NeuronIndex {
            layer: layer_index,
            neuron: neuron_index,
        };
        Ok(PyNeuroscopePage {
            object: NeuroscopeNeuronPage::from_html_str(html, neuron_index).with_context(|| {
                format!("Failed to parse html of neuroscope page for neuron {neuron_index}.")
            })?,
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
            object: NeuroscopeNeuronPage::from_file(path)
                .with_context(|| format!("Failed to read neuroscope page from file '{path:?}'."))?,
        })
    }
}

#[pyclass(name = "Database")]
struct PyDatabase {
    database: Database,
}

#[pymethods]
impl PyDatabase {
    #[staticmethod]
    fn initialize(path: &str) -> PyResult<Self> {
        let database = Runtime::new()
            .context("Failed to start async runtime to initialize database.")?
            .block_on(async { Database::initialize(path).await })?;
        Ok(PyDatabase { database })
    }

    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        let database = Runtime::new()
            .context("Failed to start async runtime to open database.")?
            .block_on(async { Database::open(path).await })?;
        Ok(PyDatabase { database })
    }

    fn scrape_neuroscope_model(&self, model_name: &str) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to scrape neuroscope.")?
            .block_on(async {
                println!("Scraping model '{model_name}' to database.");
                retrieve::neuroscope::scrape_model_to_database(&self.database, model_name).await
            })?;
        Ok(())
    }

    fn add_model_service(&self, model_name: &str, service: &str) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add model service.")?
            .block_on(async {
                println!("Adding service '{service}' to model '{model_name}'.");
                let model = self
                    .database
                    .model(model_name.to_owned())
                    .await?
                    .with_context(|| format!("Model '{model_name}' does not exist in database."))?;
                model.add_service(&self.database, service).await
            })?;
        Ok(())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_layer_to_files, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_model_to_files, m)?)?;
    m.add_function(wrap_pyfunction!(scrape_model_metadata_to_file, m)?)?;
    m.add_class::<PyNeuronViewerObject>()?;
    m.add_class::<PyNeuroscopePage>()?;
    m.add_class::<PyDatabase>()?;
    Ok(())
}
