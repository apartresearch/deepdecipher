use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::{retrieve, ModelHandle};

#[pyclass(name = "ModelHandle")]
pub struct PyModelHandle {
    pub model: ModelHandle,
}

impl PyModelHandle {
    pub fn new(model: ModelHandle) -> Self {
        Self { model }
    }
}

#[pymethods]
impl PyModelHandle {
    pub fn delete(&self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete model.")?
            .block_on(async { self.model.clone().delete().await })?;
        Ok(())
    }

    pub fn add_neuron_store(
        &mut self,
        neuron_store_path: &str,
        similarity_threshold: f32,
    ) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add neuron store.")?
            .block_on(async {
                retrieve::neuron_store::retrieve_neuron_store(
                    &mut self.model,
                    neuron_store_path,
                    similarity_threshold,
                )
                .await
            })?;
        Ok(())
    }
}
