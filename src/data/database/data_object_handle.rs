use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

use super::{data_types::DataType, Database};

#[derive(Clone)]
pub struct DataObjectHandle {
    name: String,
    data_type: DataType,
}

impl DataObjectHandle {
    pub(super) async fn new(database: &Database, data_object_name: &str) -> Result<Option<Self>> {
        const GET_DATA_OBJECT_TYPE: &str = r#"
            SELECT type, type_args
            FROM data_object
            WHERE name = $1
        "#;

        let params = (data_object_name.to_owned(),);

        let type_data: Option<(String, Vec<u8>)> = database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_DATA_OBJECT_TYPE)?;
                statement
                    .query_row(params, |row| Ok((row.get(0)?, row.get(1)?)))
                    .optional()
            })
            .await
            .with_context(|| {
                format!("Failed to get data object type for data object '{data_object_name}'.")
            })?;
        if let Some((type_name, type_args)) = type_data {
            let data_type = DataType::from_raw(type_name.as_str(), type_args.as_slice())?;
            let data_object = Self {
                name: data_object_name.to_owned(),
                data_type,
            };
            Ok(Some(data_object))
        } else {
            Ok(None)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }
}
