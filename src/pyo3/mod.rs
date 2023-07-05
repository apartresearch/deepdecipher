use pyo3::prelude::*;

mod database;
use database::PyDatabase;
mod model_handle;
use model_handle::PyModelHandle;
mod model_metadata;
use model_metadata::PyModelMetadata;

#[pyfunction]
fn setup_keyboard_interrupt() {
    crate::setup_keyboard_interrupt();
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup_keyboard_interrupt, m)?)?;
    m.add_class::<PyDatabase>()?;
    m.add_class::<PyModelHandle>()?;
    m.add_class::<PyModelMetadata>()?;
    Ok(())
}
