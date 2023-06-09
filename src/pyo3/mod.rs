use crate::server;
use pyo3::prelude::*;

#[pyfunction]
fn start_server() {
    server::start_server().unwrap();
}

/// A Python module implemented in Rust.
#[pymodule]
fn neuronav(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;

    Ok(())
}
