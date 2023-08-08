use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::{
    data::{data_types::DataType, Database},
    server::Service,
};

use super::{
    data_object_handle::PyDataObjectHandle, data_type::PyDataType, model_handle::PyModelHandle,
    model_metadata::PyModelMetadata, service_handle::PyServiceHandle,
    service_provider::PyServiceProvider,
};

#[pyclass(name = "Database")]
pub struct PyDatabase {
    pub database: Database,
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

    fn add_model(&mut self, model_metadata: &PyModelMetadata) -> PyResult<PyModelHandle> {
        let result = Runtime::new()
            .context("Failed to start async runtime to add model.")?
            .block_on(async {
                self.database
                    .add_model(model_metadata.metadata.clone())
                    .await
                    .map(PyModelHandle::new)
            })?;
        Ok(result)
    }

    fn model(&self, model_name: &str) -> PyResult<Option<PyModelHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get model.")?
            .block_on(async { self.database.model(model_name).await })?
            .map(PyModelHandle::new);
        Ok(result)
    }

    fn add_data_object(
        &mut self,
        data_object_name: &str,
        data_type: PyDataType,
    ) -> PyResult<PyDataObjectHandle> {
        match data_type.as_ref() {
            DataType::Json => {}
            data_type => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Objects of data type {data_type:?} should be added with the appropriate method.",
            )))
            }
        }
        let result = Runtime::new()
            .context("Failed to start async runtime to add data object.")?
            .block_on(async {
                self.database
                    .add_data_object(data_object_name, data_type.into())
                    .await
                    .with_context(|| format!("Failed to create data object '{data_object_name}'."))
            })
            .map(PyDataObjectHandle::new)?;

        Ok(result)
    }

    fn data_object(&self, data_object_name: &str) -> PyResult<Option<PyDataObjectHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get data object.")?
            .block_on(async { self.database.data_object(data_object_name).await })?
            .map(PyDataObjectHandle::new);
        Ok(result)
    }

    fn add_service(
        &mut self,
        name: String,
        provider: PyServiceProvider,
    ) -> PyResult<PyServiceHandle> {
        let service = Service::new(name, provider.provider);
        let result = Runtime::new()
            .context("Failed to start async runtime to add service.")?
            .block_on(async {
                self.database
                    .add_service(service)
                    .await
                    .map(PyServiceHandle::new)
            })?;
        Ok(result)
    }

    fn service(&self, service_name: &str) -> PyResult<Option<PyServiceHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get service.")?
            .block_on(async { self.database.service(service_name).await })?
            .map(PyServiceHandle::new);
        Ok(result)
    }
}
