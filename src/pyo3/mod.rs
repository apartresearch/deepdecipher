use pyo3::prelude::*;

mod database;
use database::PyDatabase;
mod model_handle;
use model_handle::PyModelHandle;
mod model_metadata;
use model_metadata::PyModelMetadata;
mod data_object_handle;
use data_object_handle::PyDataObjectHandle;
mod data_type;
use data_type::PyDataType;
mod service_provider;
use service_provider::PyServiceProvider;
mod service_handle;
use service_handle::PyServiceHandle;
mod index;
use index::PyIndex;

#[pyfunction]
fn setup_keyboard_interrupt() {
    if let Err(error) = ctrlc::set_handler(move || {
        println!("Keyboard interrupt received, exiting...");
        std::process::abort();
    }) {
        match error {
            ctrlc::Error::MultipleHandlers => {
                eprintln!("A handler already exists for keyboard interrupts.");
            }
            ctrlc::Error::NoSuchSignal(signal_type) => {
                eprintln!("Signal type not found on system: {signal_type:?}");
            }
            ctrlc::Error::System(error) => {
                eprintln!(
                    "Unexpected system error while setting keyboard interrupt handler: {error}"
                );
            }
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn deepdecipher(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup_keyboard_interrupt, m)?)?;
    m.add_class::<PyDatabase>()?;
    m.add_class::<PyModelHandle>()?;
    m.add_class::<PyModelMetadata>()?;
    m.add_class::<PyDataObjectHandle>()?;
    m.add_class::<PyDataType>()?;
    m.add_class::<PyServiceHandle>()?;
    m.add_class::<PyServiceProvider>()?;
    m.add_class::<PyIndex>()?;
    Ok(())
}
