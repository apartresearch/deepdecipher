use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

use super::{data_types::DataType, Database, Operation};

#[derive(Clone)]
pub struct DataObjectHandle {
    id: i64,
    name: String,
    data_type: DataType,
    database: Database,
}

impl DataObjectHandle {
    fn create_inner(database: Database, name: String, data_type: DataType) -> impl Operation<Self> {
        const ADD_DATA_OBJECT: &str = r#"
        INSERT INTO data_object (
            name,
            type,
            type_args
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (
            name.clone(),
            data_type.as_ref().to_owned(),
            data_type.args(),
        );

        move |transaction| {
            let id = transaction.prepare(ADD_DATA_OBJECT)?.insert(params)?;
            Ok(Self {
                id,
                name,
                data_type,
                database,
            })
        }
    }

    pub(super) async fn create(
        mut database: Database,
        name: String,
        data_type: DataType,
    ) -> Result<Self> {
        database
            .execute(Self::create_inner(
                database.clone(),
                name.clone(),
                data_type,
            ))
            .await
            .with_context(|| format!("Failed to create data object '{name}'."))
    }

    pub(super) async fn new(database: Database, data_object_name: &str) -> Result<Option<Self>> {
        const GET_DATA_OBJECT_TYPE: &str = r#"
            SELECT id, type, type_args
            FROM data_object
            WHERE name = $1
        "#;

        let params = (data_object_name.to_owned(),);

        let type_data: Option<(i64, String, Vec<u8>)> = database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_DATA_OBJECT_TYPE)?;
                statement
                    .query_row(params, |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                    .optional()
            })
            .await
            .with_context(|| {
                format!("Failed to get data object type for data object '{data_object_name}'.")
            })?;
        if let Some((id, type_name, type_args)) = type_data {
            let data_type = DataType::from_raw(type_name.as_str(), type_args.as_slice())?;
            let data_object = Self {
                id,
                name: data_object_name.to_owned(),
                data_type,
                database,
            };
            Ok(Some(data_object))
        } else {
            Ok(None)
        }
    }

    pub(super) fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    fn delete_inner(&self) -> impl Operation<()> {
        const DELETE_DATA_OBJECT_REFERENCES: &str = r#"
        DELETE FROM $DATABASE
        WHERE DELETE_DATA_OBJECT: &str = r#"
        DELETE FROM data_object_id = ?1;
        "#;
        const DELETE_DATA_OBJECT: &str = r#"
        DELETE FROM data_object
        WHERE id = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 4] = [
            "model_data",
            "layer_data",
            "neuron_data",
            "model_data_object",
        ];

        let params = (self.id,);
        move |transaction| {
            for table in REFERENCE_TABLES.iter() {
                let mut statement = transaction.prepare(
                    DELETE_DATA_OBJECT_REFERENCES
                        .replace("$DATABASE", table)
                        .as_str(),
                )?;
                statement.execute(params)?;
            }
            transaction.prepare(DELETE_DATA_OBJECT)?.execute(params)?;
            Ok(())
        }
    }

    pub async fn delete(mut self) -> Result<()> {
        self.database
            .execute(self.delete_inner())
            .await
            .with_context(|| format!("Problem deleting data object '{name}'.", name = self.name()))
    }
}
