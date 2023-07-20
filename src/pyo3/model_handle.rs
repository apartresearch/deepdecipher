use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::{retrieve, ModelHandle};

use super::{data_object_handle::PyDataObjectHandle, service_handle::PyServiceHandle};

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

    pub fn add_neuron2graph_graphs(&mut self, neuron2graph_path: &str) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add neuron2graph graphs.")?
            .block_on(async {
                retrieve::neuron2graph::retrieve_neuron2graph(&mut self.model, neuron2graph_path)
                    .await
            })?;
        Ok(())
    }

    pub fn has_data_object(&self, data_object: &PyDataObjectHandle) -> PyResult<bool> {
        let result = Runtime::new()
            .context("Failed to start async runtime to check whether model has data object.")?
            .block_on(async { self.model.has_data_object(&data_object.data_object).await })?;
        Ok(result)
    }

    pub fn delete_data_object(&mut self, data_object: &PyDataObjectHandle) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete model.")?
            .block_on(async {
                self.model
                    .delete_data_object(&data_object.data_object)
                    .await
            })?;
        Ok(())
    }

    pub fn has_service(&self, service: &PyServiceHandle) -> PyResult<bool> {
        let result = Runtime::new()
            .context("Failed to start async runtime to check whether model has service.")?
            .block_on(async { self.model.has_service(&service.service_handle).await })?;
        Ok(result)
    }

    pub fn add_service(&mut self, service: &PyServiceHandle) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add service.")?
            .block_on(async { self.model.add_service(&service.service_handle).await })?;
        Ok(())
    }
}
