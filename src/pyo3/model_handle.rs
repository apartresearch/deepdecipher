use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::ModelHandle;

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
}
