use crate::{
    data::{database::Database, retrieve},
    server,
};
use anyhow::Context;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyfunction]
fn start_server() {
    server::start_server().unwrap();
}

#[pyclass(name = "Database")]
struct PyDatabase {
    database: Database,
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

    fn scrape_neuroscope_model(&self, model_name: &str) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to scrape neuroscope.")?
            .block_on(async {
                let model = if let Some(model) = self.database.model(model_name.to_owned()).await? {
                    model
                } else {
                    let metadata = retrieve::neuroscope::scrape_model_metadata(model_name).await?;
                    self.database.add_model(metadata).await?
                };
                println!("Scraping model '{model_name}' to database.");
                retrieve::neuroscope::scrape_model_to_database(&self.database, &model).await
            })?;
        Ok(())
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

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_class::<PyDatabase>()?;
    Ok(())
}
