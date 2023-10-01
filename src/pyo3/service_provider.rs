use pyo3::prelude::*;

use crate::server::ServiceProvider;

use super::data_type_handle::PyDataTypeHandle;

#[pyclass(name = "ServiceProvider")]
#[derive(Clone)]
pub struct PyServiceProvider {
    pub provider: ServiceProvider,
}

#[pymethods]
impl PyServiceProvider {
    #[staticmethod]
    pub fn neuroscope() -> Self {
        PyServiceProvider {
            provider: ServiceProvider::Neuroscope,
        }
    }

    #[staticmethod]
    pub fn neuron_explainer() -> Self {
        PyServiceProvider {
            provider: ServiceProvider::NeuronExplainer,
        }
    }

    #[staticmethod]
    pub fn neuron2graph() -> Self {
        PyServiceProvider {
            provider: ServiceProvider::Neuron2Graph,
        }
    }

    #[staticmethod]
    pub fn neuron2graph_search() -> Self {
        PyServiceProvider {
            provider: ServiceProvider::Neuron2GraphSearch,
        }
    }

    #[staticmethod]
    pub fn json(data_type: &PyDataTypeHandle) -> Self {
        PyServiceProvider {
            provider: ServiceProvider::json(data_type.data_type.name().to_owned()),
        }
    }

    pub fn __repr__(&self) -> &str {
        self.provider.as_ref()
    }
}
