use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::ServiceHandle;

#[pyclass(name = "ServiceHandle")]
pub struct PyServiceHandle {
    pub service_handle: ServiceHandle,
}

impl PyServiceHandle {
    pub fn new(service_handle: ServiceHandle) -> Self {
        Self { service_handle }
    }
}

#[pymethods]
impl PyServiceHandle {
    pub fn delete(&self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete service.")?
            .block_on(async { self.service_handle.clone().delete().await })?;
        Ok(())
    }
}
