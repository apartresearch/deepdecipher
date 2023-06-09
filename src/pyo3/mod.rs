use crate::{data::NeuronViewerObject, server};
use anyhow::Context;
use pyo3::prelude::*;

#[pyfunction]
fn start_server() {
    server::start_server().unwrap();
}

#[pyclass(name = "NeuronViewerObject")]
struct PyNeuronViewerObject {
    object: NeuronViewerObject,
}

#[pymethods]
impl PyNeuronViewerObject {
    #[new]
    fn new(json: &str) -> PyResult<Self> {
        let json_value = serde_json::from_str(json).context("failed to parse json")?;
        Ok(PyNeuronViewerObject {
            object: NeuronViewerObject::from_json(&json_value)?,
        })
    }

    fn to_file(&self, path: &str) {
        self.object.to_file(path);
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_class::<PyNeuronViewerObject>()?;
    Ok(())
}
