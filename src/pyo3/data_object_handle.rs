use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::DataObjectHandle;

#[pyclass(name = "DataObjectHandle")]
pub struct PyDataObjectHandle {
    pub data_object: DataObjectHandle,
}

impl PyDataObjectHandle {
    pub fn new(data_object: DataObjectHandle) -> Self {
        Self { data_object }
    }
}

#[pymethods]
impl PyDataObjectHandle {
    pub fn delete(&self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete data object.")?
            .block_on(async { self.data_object.clone().delete().await })?;
        Ok(())
    }
}
