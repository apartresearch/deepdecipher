use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::{
    data::{retrieve, Database},
    server,
};

use super::{model_handle::PyModelHandle, model_metadata::PyModelMetadata};

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

    fn start_server(&self) -> PyResult<()> {
        server::start_server(self.database.clone())?;
        Ok(())
    }

    fn add_model(&self, model_metadata: &PyModelMetadata) -> PyResult<PyModelHandle> {
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

    fn model(&self, model_name: &str) -> PyResult<PyModelHandle> {
        let result = Runtime::new()
            .context("Failed to start async runtime to add model.")?
            .block_on(async { self.database.model(model_name).await })?
            .map(PyModelHandle::new)
            .with_context(|| format!("Database has no model named '{model_name}'."))?;
        Ok(result)
    }

    fn scrape_neuroscope_model(&self, model_name: &str) -> PyResult<PyModelHandle> {
        let result = Runtime::new()
            .context("Failed to start async runtime to scrape neuroscope.")?
            .block_on(async {
                let model = if let Some(model) = self.database.model(model_name.to_owned()).await.with_context(|| format!("Failed to get model '{model_name}' from database."))? {
                    model
                } else {
                    let metadata = retrieve::neuroscope::scrape_model_metadata(model_name).await.with_context(|| format!("Failed to scrape metadata for model '{model_name}' from Neuroscope."))?;
                    self.database.add_model(metadata).await.context("Failed to add model to database.")?
                };
                println!("Scraping model '{model_name}' to database.");
                retrieve::neuroscope::scrape_model_to_database(&self.database, &model).await.with_context(|| format!("Failed to scrape data for model '{model_name}' from Neuroscope."))?;
                anyhow::Ok(model)
            })
            .with_context(|| format!("Failed to scrape neuroscope model '{model_name}'."))
            .map(PyModelHandle::new)?;
        Ok(result)
    }

    fn add_model_service(&self, model_name: &str, service_name: &str) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add model service.")?
            .block_on(async {
                let model = self
                    .database
                    .model(model_name.to_owned())
                    .await?
                    .with_context(|| format!("Model '{model_name}' does not exist in database."))?;
                let service = self
                    .database
                    .service(service_name.to_owned())
                    .await?
                    .with_context(|| {
                        format!("Service '{service_name}' does not exist in database.")
                    })?;
                println!("Adding service '{service_name}' to model '{model_name}'...");
                model.add_service(&service).await
            })?;
        Ok(())
    }
}
