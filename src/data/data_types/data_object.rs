use std::str::FromStr;

use async_trait::async_trait;

use anyhow::{ensure, Context, Result};
use strum::{AsRefStr, EnumDiscriminants, EnumString};

use crate::data::database::Database;

use super::Neuroscope;

#[derive(Clone, Debug, AsRefStr, EnumString, EnumDiscriminants, PartialEq, Eq)]
#[strum_discriminants(derive(EnumString))]
pub enum DataType {
    Neuroscope,
}

impl DataType {
    pub fn from_raw(data_type: &str, type_args: &[u8]) -> Result<Self> {
        match DataTypeDiscriminants::from_str(data_type)
            .with_context(|| format!("Unexpected data type '{data_type}'."))?
        {
            DataTypeDiscriminants::Neuroscope => {
                ensure!(
                    type_args.is_empty(),
                    "Neuroscope data objects do not take type arguments."
                );
                Ok(DataType::Neuroscope)
            }
        }
    }

    pub fn args(&self) -> Vec<u8> {
        match self {
            Self::Neuroscope => Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct DataObject {
    data_type: DataType,
}

impl DataObject {
    pub async fn new(database: &Database, data_object_name: &str) -> Result<Self> {
        const GET_DATA_OBJECT_TYPE: &str = r#"
            SELECT type, type_args
            FROM data_object
            WHERE name = $1
        "#;

        let params = (data_object_name.to_owned(),);
        let (data_type, type_args): (String, Vec<u8>) = database
            .connection()
            .call(|connection| {
                let mut statement = connection.prepare(GET_DATA_OBJECT_TYPE)?;
                statement.query_row(params, |row| Ok((row.get(0)?, row.get(1)?)))
            })
            .await
            .with_context(|| format!("Failed to create data object '{data_object_name}'."))?;

        match DataTypeDiscriminants::from_str(data_type.as_str())
            .with_context(|| format!("Unexpected data type '{data_type}'."))?
        {
            DataTypeDiscriminants::Neuroscope => {
                ensure!(
                    type_args.is_empty(),
                    "Neuroscope data objects do not take type arguments."
                );
                Ok(Self {
                    data_type: DataType::Neuroscope,
                })
            }
        }
    }

    pub fn neuroscope(&self) -> Option<Neuroscope> {
        if self.data_type == DataType::Neuroscope {
            Some(Neuroscope)
        } else {
            None
        }
    }
}
