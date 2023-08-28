use crate::{
    data::{data_types::DataType, json::JsonData, DataObjectHandle, ModelHandle},
    Index,
};

use anyhow::{bail, Context, Result};

pub async fn store_json_data(
    model_handle: &mut ModelHandle,
    data_object_handle: &DataObjectHandle,
    index: Index,
    json: serde_json::Value,
) -> Result<()> {
    let model_name = model_handle.name();
    let data_object_name = data_object_handle.name();
    match data_object_handle.data_type() {
        DataType::Json => {}
        _ => bail!("Data object must have type JSON."),
    }
    let data = JsonData::new(json);
    model_handle
        .add_data(
            data_object_handle,
            index,
            data.to_binary().with_context(||
                format!("Failed to serialize JSON data of data object '{data_object_name}' for {index} in model '{model_name}'.", index = index.error_string())
            )?,
        )
        .await
}
