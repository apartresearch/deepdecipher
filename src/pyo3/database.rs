use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::{
    data::{data_types::DataType, Database},
    server::Service,
};

use super::{
    data_type::PyDataType, data_type_handle::PyDataTypeHandle, model_handle::PyModelHandle,
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

    fn models(&self) -> PyResult<Vec<PyModelHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get models.")?
            .block_on(async {
                self.database
                    .all_models()
                    .await
                    .map(|models| models.into_iter().map(PyModelHandle::new).collect())
            })?;
        Ok(result)
    }

    fn add_data_type(
        &mut self,
        data_type_name: &str,
        data_type: PyDataType,
    ) -> PyResult<PyDataTypeHandle> {
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
                    .add_data_type(data_type_name, data_type.into())
                    .await
                    .with_context(|| format!("Failed to create data object '{data_type_name}'."))
            })
            .map(PyDataTypeHandle::new)?;

        Ok(result)
    }

    fn data_type(&self, data_type_name: &str) -> PyResult<Option<PyDataTypeHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get data object.")?
            .block_on(async { self.database.data_type(data_type_name).await })?
            .map(PyDataTypeHandle::new);
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

    fn services(&self) -> PyResult<Vec<PyServiceHandle>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to get services.")?
            .block_on(async {
                let service_names = self.database.all_service_names().await?;
                let mut result = Vec::with_capacity(service_names.len());
                for service_name in service_names {
                    let service_handle = self
                        .database
                        .service(&service_name)
                        .await?
                        .expect("We just got the name from the database, so it must exist.");
                    result.push(PyServiceHandle::new(service_handle));
                }
                anyhow::Ok(result)
            })?;
        Ok(result)
    }
}
