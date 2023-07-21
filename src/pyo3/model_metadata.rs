use pyo3::prelude::*;

use crate::data::Metadata;

#[pyclass(name = "ModelMetadata")]
pub struct PyModelMetadata {
    pub metadata: Metadata,
}

#[pymethods]
impl PyModelMetadata {
    #[new]
    fn new(
        name: String,
        num_layers: u32,
        layer_size: u32,
        activation_function: String,
        num_total_parameters: u32,
        dataset: String,
    ) -> Self {
        Self {
            metadata: Metadata {
                name,
                num_layers,
                layer_size,
                activation_function,
                num_total_neurons: num_layers * layer_size,
                num_total_parameters,
                dataset,
            },
        }
    }

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
