use anyhow::{bail, Context, Result};

use crate::{
    data::{
        data_objects::{DataObject, JsonData},
        data_types::DataType,
        DataTypeHandle, ModelHandle,
    },
    Index,
};

pub async fn store_json_data(
    model_handle: &mut ModelHandle,
    data_type_handle: &DataTypeHandle,
    index: Index,
    json: serde_json::Value,
) -> Result<()> {
    let model_name = model_handle.name();
    let data_type_name = data_type_handle.name();
    match data_type_handle.data_type() {
        DataType::Json => {}
        _ => bail!("Data object must have type JSON."),
    }
    let data = JsonData::new(json);
    model_handle
        .add_data(
            data_type_handle,
            index,
            data.to_binary().with_context(|| {
                format!(
                    "Failed to serialize JSON data of data object '{data_type_name}' for {index} \
                     in model '{model_name}'.",
                    index = index.error_string()
                )
            })?,
        )
        .await
}
