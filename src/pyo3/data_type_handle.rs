use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::DataTypeHandle;

#[pyclass(name = "DataTypeHandle")]
pub struct PyDataTypeHandle {
    pub data_type: DataTypeHandle,
}

impl PyDataTypeHandle {
    pub fn new(data_type: DataTypeHandle) -> Self {
        Self { data_type }
    }
}

#[pymethods]
impl PyDataTypeHandle {
    pub fn delete(&self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete data object.")?
            .block_on(async { self.data_type.clone().delete().await })?;
        Ok(())
    }
}
