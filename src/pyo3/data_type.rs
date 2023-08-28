use pyo3::prelude::*;

use crate::data::data_types::DataType;

#[pyclass(name = "DataType")]
#[derive(Clone, Debug)]
pub struct PyDataType {
    pub data_type: DataType,
}

impl PyDataType {
    pub fn new(data_type: DataType) -> Self {
        Self { data_type }
    }
}

impl From<PyDataType> for DataType {
    fn from(py_data_type: PyDataType) -> Self {
        py_data_type.data_type
    }
}

impl From<DataType> for PyDataType {
    fn from(data_type: DataType) -> Self {
        Self::new(data_type)
    }
}

impl AsRef<DataType> for PyDataType {
    fn as_ref(&self) -> &DataType {
        &self.data_type
    }
}

#[pymethods]
impl PyDataType {
    #[staticmethod]
    pub fn json() -> Self {
        Self::new(DataType::Json)
    }

    pub fn __repr__(&self) -> String {
        format!("{data_type:?}", data_type = self.data_type)
    }
}
