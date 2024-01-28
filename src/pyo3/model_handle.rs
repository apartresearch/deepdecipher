use anyhow::{bail, Context};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use super::{
    data_type_handle::PyDataTypeHandle, index::PyIndex, model_metadata::PyModelMetadata,
    service_handle::PyServiceHandle,
};
use crate::data::{retrieve, ModelHandle};

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
        PyModelMetadata {
            metadata: self.model.metadata().clone(),
        }
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
                retrieve::neuroscope::scrape_model_to_database(model)
                    .await
                    .with_context(|| {
                        format!("Failed to scrape data for model '{model_name}' from Neuroscope.")
                    })?;
                anyhow::Ok(model)
            })
            .with_context(|| format!("Failed to scrape neuroscope model '{model_name}'."))?;
        Ok(())
    }

    pub fn scrape_missing_neuroscope_items(&mut self) -> PyResult<()> {
        let model_name = self.model.name().to_owned();
        Runtime::new()
            .context("Failed to start async runtime to scrape neuroscope.")?
            .block_on(async {
                let model = &mut self.model;
                let data_type = model
                    .database()
                    .data_type("neuroscope")
                    .await?
                    .with_context(|| {
                        format!("No 'neuroscope' data object for model '{model_name}'")
                    })?;
                let mut missing_indices: Vec<_> =
                    model.missing_neuron_items(&data_type).await?.collect();
                while !missing_indices.is_empty() {
                    println!(
                        "{} missing items for model '{model_name}'. Scraping...",
                        missing_indices.len(),
                        model_name = model.name()
                    );
                    retrieve::neuroscope::scrape_indices_to_database(
                        model,
                        &data_type,
                        missing_indices.iter().copied(),
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to scrape data for model '{model_name}' from Neuroscope.")
                    })?;
                    missing_indices = model.missing_neuron_items(&data_type).await?.collect();
                }
                anyhow::Ok(())
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
                retrieve::neuron_explainer::retrieve_neuron_explainer_small(&mut self.model).await
            })?;
        Ok(())
    }

    pub fn add_neuron_explainer_xl(&mut self) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add neuron explainer.")?
            .block_on(async {
                retrieve::neuron_explainer::retrieve_neuron_explainer_xl(&mut self.model).await
            })?;
        Ok(())
    }

    pub fn add_json_data(
        &mut self,
        data_type: &PyDataTypeHandle,
        index: PyIndex,
        json_data: &str,
    ) -> PyResult<()> {
        let model = &mut self.model;
        let data_type = &data_type.data_type;
        let json = serde_json::from_str(json_data).context("Failed to parse JSON data.")?;
        Runtime::new()
            .context("Failed to start async runtime to add JSON data.")?
            .block_on(async {
                if !model.has_data_type(data_type).await.with_context(|| {
                    format!(
                        "Failed to check whether model '{model_name}' has data object \
                         '{data_type_name}'.",
                        model_name = model.name(),
                        data_type_name = data_type.name()
                    )
                })? {
                    bail!(
                        "Cannot add JSON data to data object '{data_type_name}' for {index} in \
                         model '{model_name}' because model does not have data object.",
                        data_type_name = data_type.name(),
                        index = index.index.error_string(),
                        model_name = model.name()
                    )
                }
                retrieve::json::store_json_data(model, data_type, index.into(), json)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to add JSON data to '{data_type_name}' for {index} in model \
                             '{model_name}'.",
                            data_type_name = data_type.name(),
                            index = index.index.error_string(),
                            model_name = model.name()
                        )
                    })
            })?;
        Ok(())
    }

    pub fn add_data_type(&mut self, data_type: &PyDataTypeHandle) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to add data object to model.")?
            .block_on(async {
                self.model
                    .add_data_type(&data_type.data_type)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to add data object '{data_type_name}' to model '{model_name}'.",
                            data_type_name = data_type.data_type.name(),
                            model_name = self.model.name()
                        )
                    })
            })?;
        Ok(())
    }

    pub fn has_data_type(&self, data_type: &PyDataTypeHandle) -> PyResult<bool> {
        let result = Runtime::new()
            .context("Failed to start async runtime to check whether model has data object.")?
            .block_on(async { self.model.has_data_type(&data_type.data_type).await })?;
        Ok(result)
    }

    pub fn delete_data_type(&mut self, data_type: &PyDataTypeHandle) -> PyResult<()> {
        Runtime::new()
            .context("Failed to start async runtime to delete model.")?
            .block_on(async { self.model.delete_data_type(&data_type.data_type).await })?;
        Ok(())
    }

    pub fn missing_data_types(&self, service: &PyServiceHandle) -> PyResult<Vec<String>> {
        let result = Runtime::new()
            .context("Failed to start async runtime to check missing data objects for service.")?
            .block_on(async { self.model.missing_data_types(&service.service_handle).await })?;
        Ok(result)
    }

    pub fn has_service(&self, service: &PyServiceHandle) -> PyResult<bool> {
        self.missing_data_types(service)
            .map(|missing| missing.is_empty())
    }
}
