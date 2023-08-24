use anyhow::{Context, bail};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::data::{retrieve, ModelHandle};

use super::{
    data_object_handle::PyDataObjectHandle, index::PyIndex, service_handle::PyServiceHandle, model_metadata::PyModelMetadata,
};

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
    pub fn metadata(&self) -> PyModelMetadata {
        PyModelMetadata{ metadata: self.model.metadata().clone() }
    }

    pub fn name(&self) -> &str {
        self.model.name()
    }

    pub fn delete(&self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete model.")?
            .block_on(async { self.model.clone().delete().await })?;
        Ok(())
    }

    pub fn scrape_neuroscope_model(&mut self) -> PyResult<()> {
        let model_name = self.model.name().to_owned();
        Runtime::new()
            .context("Failed to start async runtime to scrape neuroscope.")?
            .block_on(async {
                let model = &mut self.model;
                println!("Scraping model '{model_name}' to database.");
                retrieve::neuroscope::scrape_model_to_database(model).await.with_context(|| format!("Failed to scrape data for model '{model_name}' from Neuroscope."))?;
                anyhow::Ok(model)
            })
            .with_context(|| format!("Failed to scrape neuroscope model '{model_name}'."))?;
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

    pub fn add_neuron_explainer_small(&mut self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add neuron explainer.")?
            .block_on(async {
                retrieve::neuron_explainer::retrieve_neuron_explainer_small(&mut self.model)
                    .await
            })?;
        Ok(())
    }

    pub fn add_neuron_explainer_xl(&mut self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add neuron explainer.")?
            .block_on(async {
                retrieve::neuron_explainer::retrieve_neuron_explainer_xl(&mut self.model)
                    .await
            })?;
        Ok(())
    }

    pub fn add_json_data(
        &mut self,
        data_object: &PyDataObjectHandle,
        index: PyIndex,
        json_data: &str,
    ) -> PyResult<()> {
        let model = &mut self.model;
        let data_object = &data_object.data_object;
        let json = serde_json::from_str(json_data).context("Failed to parse JSON data.")?;
        Runtime::new()
            .context("Failed to start async runtime to add JSON data.")?
            .block_on(async {
                if !model.has_data_object(data_object).await.with_context(|| 
                    format!(
                        "Failed to check whether model '{model_name}' has data object '{data_object_name}'.", 
                        model_name=model.name(), 
                        data_object_name=data_object.name()
                    )
                )? {
                    bail!("Cannot add JSON data to data object '{data_object_name}' for {index} in model '{model_name}' because model does not have data object.", 
                        data_object_name=data_object.name(), 
                        index=index.index.error_string(), 
                        model_name=model.name())
                }
                retrieve::json::store_json_data(
                    model,
                    data_object,
                    index.into(),
                    json,
                )
                .await.with_context(|| format!(
                    "Failed to add JSON data to '{data_object_name}' for {index} in model '{model_name}'.", 
                    data_object_name=data_object.name(), 
                    index=index.index.error_string(), 
                    model_name=model.name()
                ))
            })?;
        Ok(())
    }

    pub fn add_data_object(&mut self, data_object: &PyDataObjectHandle) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add data object to model.")?
            .block_on(async { 
                self.model.add_data_object(&data_object.data_object).await.with_context(|| format!("Failed to add data object '{data_object_name}' to model '{model_name}'.", 
                    data_object_name=data_object.data_object.name(), 
                    model_name=self.model.name()))
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

    pub fn missing_data_objects(&self, service: &PyServiceHandle) -> PyResult<Vec<String>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to check missing data objects for service.")?
            .block_on(async {
                self.model
                    .missing_data_objects(&service.service_handle)
                    .await
            })?;
        Ok(result)
    }

    pub fn has_service(&self, service: &PyServiceHandle) -> PyResult<bool> {
        self.missing_data_objects(service)
            .map(|missing| missing.is_empty())
    }
}
