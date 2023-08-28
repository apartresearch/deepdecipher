use pyo3::prelude::*;

use crate::Index;

#[pyclass(name = "Index")]
#[derive(Clone, Copy, Debug)]
pub struct PyIndex {
    pub index: Index,
}

impl PyIndex {
    pub fn new(index: Index) -> Self {
        Self { index }
    }
}

impl From<Index> for PyIndex {
    fn from(index: Index) -> Self {
        Self::new(index)
    }
}

impl From<PyIndex> for Index {
    fn from(py_index: PyIndex) -> Self {
        py_index.index
    }
}

#[pymethods]
impl PyIndex {
    #[staticmethod]
    pub fn model() -> Self {
        Self::new(Index::Model)
    }

    #[staticmethod]
    pub fn layer(layer_index: u32) -> Self {
        Self::new(Index::Layer(layer_index))
    }

    #[staticmethod]
    pub fn neuron(layer_index: u32, neuron_index: u32) -> Self {
        Self::new(Index::Neuron(layer_index, neuron_index))
    }

    pub fn __repr__(&self) -> String {
        format!("{index:?}", index = self.index)
    }
}
