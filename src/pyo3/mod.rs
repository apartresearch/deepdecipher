use crate::{
    data::{NeuronViewerObject, NeuroscopePage},
    server,
};
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

#[pyclass(name = "NeuroscopePage")]
struct PyNeuroscopePage {
    object: NeuroscopePage,
}

#[pymethods]
impl PyNeuroscopePage {
    #[new]
    fn new(html: &str, neuron_index: u32, layer_index: u32) -> PyResult<Self> {
        Ok(PyNeuroscopePage {
            object: NeuroscopePage::from_html_str(html, neuron_index, layer_index)
                .with_context(|| format!("Failed to parse html of neuroscope page for neuron index {neuron_index} on layer {layer_index}."))?,
        })
    }

    fn __str__(&self) -> String {
        format!("{:?}", self.object)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    m.add_class::<PyNeuronViewerObject>()?;
    m.add_class::<PyNeuroscopePage>()?;
    Ok(())
}
