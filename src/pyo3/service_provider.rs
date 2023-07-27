use pyo3::prelude::*;

use crate::server::ServiceProvider;

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
    pub fn json(data_object_name: String) -> Self {
        PyServiceProvider {
            provider: ServiceProvider::json(data_object_name),
        }
    }

    pub fn __repr__(&self) -> &str {
        self.provider.as_ref()
    }
}
