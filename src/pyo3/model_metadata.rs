use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::{retrieve, Metadata};

#[pyclass(name = "ModelMetadata")]
pub struct PyModelMetadata {
    pub metadata: Metadata,
}

#[pymethods]
impl PyModelMetadata {
    #[new]
    fn new(
        name: String,
        num_layers: u32,
        layer_size: u32,
        activation_function: String,
        num_total_parameters: u32,
        dataset: String,
    ) -> Self {
        Self {
            metadata: Metadata {
                name,
                num_layers,
                layer_size,
                activation_function,
                num_total_neurons: num_layers * layer_size,
                num_total_parameters,
                dataset,
            },
        }
    }

    #[staticmethod]
    fn from_neuroscope(model_name: &str) -> PyResult<Self> {
        let metadata = Runtime::new()
            .context("Failed to start async runtime to scrape metadata from neuroscope.")?
            .block_on(async {
                retrieve::neuroscope::scrape_model_metadata(model_name)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to scrape metadata for model '{model_name}' from Neuroscope."
                        )
                    })
            })?;
        Ok(Self { metadata })
    }

    #[getter]
    fn name(&self) -> String {
        self.metadata.name.clone()
    }

    #[getter]
    fn activation_function(&self) -> String {
        self.metadata.activation_function.clone()
    }

    #[getter]
    fn num_total_neurons(&self) -> u32 {
        self.metadata.num_total_neurons
    }

    #[getter]
    fn num_total_parameters(&self) -> u32 {
        self.metadata.num_total_parameters
    }

    #[getter]
    fn num_layers(&self) -> u32 {
        self.metadata.num_layers
    }

    #[getter]
    fn layer_size(&self) -> u32 {
        self.metadata.layer_size
    }

    #[getter]
    fn dataset(&self) -> String {
        self.metadata.dataset.clone()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.metadata)
    }
}
