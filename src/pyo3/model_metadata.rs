use pyo3::prelude::*;

use crate::data::Metadata;

#[pyclass(name = "ModelMetadata")]
pub struct PyModelMetadata {
    pub metadata: Metadata,
}

impl PyModelMetadata {
    pub fn new(model_metadata: Metadata) -> Self {
        Self {
            metadata: model_metadata,
        }
    }
}

#[pymethods]
impl PyModelMetadata {
    #[getter]
    fn name(&self) -> String {
        self.metadata.name.clone()
    }

    #[getter]
    fn activation_function(&self) -> String {
        self.metadata.activation_function.clone()
    }

    #[getter]
    fn num_total_neurons(&self) -> u32 {
        self.metadata.num_total_neurons
    }

    #[getter]
    fn num_total_parameters(&self) -> u32 {
        self.metadata.num_total_parameters
    }

    #[getter]
    fn dataset(&self) -> String {
        self.metadata.dataset.clone()
    }
}
