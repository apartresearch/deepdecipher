use std::path::Path;

use anyhow::{bail, Context, Result};
use rusqlite::OptionalExtension;
use tokio_rusqlite::Connection;

use crate::data::LayerMetadata;

use super::{
    data_types::{DataObject, DataType},
    Metadata,
};

const MODEL_TABLE: &str = r#"
CREATE TABLE model (
    name                    TEXT PRIMARY KEY,
    num_layers              INTEGER NOT NULL,
    neurons_per_layer       INTEGER NOT NULL,
    activation_function     TEXT NOT NULL,
    num_total_parameters    INTEGER NOT NULL,
    dataset                 TEXT NOT NULL
    CHECK (num_layers >= 0 AND neurons_per_layer >= 0 AND num_total_parameters >= 0)
  ) STRICT;
"#;

const SERVICE_TABLE: &str = r#"
CREATE TABLE service (
    name                    TEXT PRIMARY KEY,
    provider                TEXT NOT NULL,
    provider_args           BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_SERVICE_TABLE: &str = r#"
CREATE TABLE model_service (
    model                   TEXT NOT NULL,
    service                 TEXT NOT NULL,
    FOREIGN KEY(model) REFERENCES model(name),
    FOREIGN KEY(service) REFERENCES service(name)
  ) STRICT;
"#;

const DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE data_object (
    name                    TEXT PRIMARY KEY,
    type                    TEXT NOT NULL,
    type_args               BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE model_data_object (
    model                   TEXT NOT NULL,
    data_object             TEXT NOT NULL,
    FOREIGN KEY(model) REFERENCES model(name),
    FOREIGN KEY(data_object) REFERENCES data_object(name)
)
"#;

const MODEL_DATA_TABLE: &str = r#"
CREATE TABLE model_data (
    model                   TEXT NOT NULL,
    data_object             TEXT NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_object),
    FOREIGN KEY(model) REFERENCES model(name)
    FOREIGN KEY(data_object) REFERENCES data_object(name)
  ) STRICT;
"#;

const LAYER_DATA_TABLE: &str = r#"
CREATE TABLE layer_data (
    model                   TEXT NOT NULL,
    data_object             TEXT NOT NULL,
    layer_index             INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_object, layer_index),
    FOREIGN KEY(model) REFERENCES model(name),
    FOREIGN KEY(data_object) REFERENCES data_object(name)
    CHECK (layer_index >= 0)
  ) STRICT;
"#;

const NEURON_DATA_TABLE: &str = r#"
CREATE TABLE neuron_data (
    model                   TEXT NOT NULL,
    data_object             TEXT NOT NULL,
    layer_index             INTEGER NOT NULL,
    neuron_index            INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_object, layer_index, neuron_index),
    FOREIGN KEY(model) REFERENCES model(name),
    FOREIGN KEY(data_object) REFERENCES data_object(name)
    CHECK (layer_index >= 0 AND neuron_index >= 0)
  ) STRICT;
"#;

const TABLES: [&str; 8] = [
    MODEL_TABLE,
    SERVICE_TABLE,
    MODEL_SERVICE_TABLE,
    DATA_OBJECT_TABLE,
    MODEL_DATA_OBJECT_TABLE,
    MODEL_DATA_TABLE,
    LAYER_DATA_TABLE,
    NEURON_DATA_TABLE,
];

#[derive(Clone)]
pub struct Database {
    connection: Connection,
}

impl Database {
    pub async fn initialize(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().exists() {
            bail!("Database already exists at {:?}", path.as_ref())
        }

        let database = Connection::open(path).await?;

        let database = Database {
            connection: database,
        };

        for table in TABLES.iter() {
            database
                .connection
                .call(|connection| connection.execute(table, ()))
                .await?;
        }

        database.add_service("metadata", "metadata", vec![]).await?;

        Ok(database)
    }

    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        if !path.as_ref().exists() {
            bail!("Database does not exist at {:?}", path.as_ref())
        }

        let database = Connection::open(path).await?;

        Ok(Database {
            connection: database,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub async fn add_model(&self, metadata: Metadata) -> Result<ModelHandle> {
        ModelHandle::create(self, metadata).await
    }

    pub async fn model(&self, model_name: String) -> Result<Option<ModelHandle>> {
        const GET_MODEL: &str = r#"
        SELECT
            name,
            num_layers,
            neurons_per_layer,
            activation_function,
            num_total_parameters,
            dataset
        FROM model
        WHERE name = ?1;
        "#;

        let metadata = self
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_MODEL)?;
                let mut rows = statement.query((model_name,))?;

                let row = if let Some(row) = rows.next()? {
                    row
                } else {
                    return Ok(None);
                };

                let num_layers: u32 = row.get(1)?;
                let neurons_per_layer = row.get(2)?;
                let layers = vec![
                    LayerMetadata {
                        num_neurons: neurons_per_layer
                    };
                    num_layers as usize
                ];

                Ok(Some(Metadata {
                    name: row.get(0)?,
                    layers,
                    activation_function: row.get(3)?,
                    num_total_neurons: num_layers * neurons_per_layer,
                    num_total_parameters: row.get(4)?,
                    dataset: row.get(5)?,
                }))
            })
            .await?;

        Ok(metadata.map(|metadata| ModelHandle { metadata }))
    }

    pub async fn delete_model(&self, model_name: String) -> Result<()> {
        const DELETE_MODEL_REFERENCES: &str = r#"
        DELETE FROM $DATABASE
        WHERE model = ?1;
        "#;
        const DELETE_MODEL: &str = r#"
        DELETE FROM model
        WHERE name = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 5] = [
            "model_service",
            "model_data_object",
            "model_data",
            "layer_data",
            "neuron_data",
        ];

        for table in REFERENCE_TABLES.iter() {
            let params = (model_name.clone(),);
            self.connection
                .call(|connection| {
                    let mut statement = connection
                        .prepare(DELETE_MODEL_REFERENCES.replace("$DATABASE", table).as_str())?;
                    statement.execute(params)?;
                    Ok(())
                })
                .await?;
        }
        let params = (model_name.clone(),);
        self.connection
            .call(|connection| {
                let mut statement = connection.prepare(DELETE_MODEL)?;
                statement.execute(params)?;
                Ok(())
            })
            .await
            .context("Problem deleting model '{model_name}'.")
    }

    pub async fn add_service(
        &self,
        service_name: impl AsRef<str>,
        provider: impl AsRef<str>,
        provider_args: impl AsRef<[u8]>,
    ) -> Result<()> {
        const ADD_SERVICE: &str = r#"
        INSERT INTO service (
            name,
            provider,
            provider_args
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (
            service_name.as_ref().to_owned(),
            provider.as_ref().to_owned(),
            provider_args.as_ref().to_vec(),
        );

        self.connection
            .call(|connection| connection.execute(ADD_SERVICE, params))
            .await?;

        Ok(())
    }

    pub async fn add_data_object(
        &self,
        data_object_name: impl AsRef<str>,
        data_type: DataType,
    ) -> Result<()> {
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
            data_object_name.as_ref().to_owned(),
            data_type.as_ref().to_owned(),
            data_type.args(),
        );

        self.connection
            .call(|connection| connection.execute(ADD_DATA_OBJECT, params))
            .await?;

        Ok(())
    }

    pub async fn data_object_type(
        &self,
        data_object_name: impl AsRef<str>,
    ) -> Result<Option<DataType>> {
        const GET_DATA_OBJECT_TYPE: &str = r#"
        SELECT
            type,
            type_args
        FROM data_object
        WHERE name = ?1;
        "#;

        let data_object_name = data_object_name.as_ref();

        let params = (data_object_name.to_owned(),);
        let type_data: Option<(String, Vec<u8>)> = self
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
            Ok(Some(DataType::from_raw(
                type_name.as_str(),
                type_args.as_slice(),
            )?))
        } else {
            Ok(None)
        }
    }
}

pub struct ModelHandle {
    metadata: Metadata,
}

impl ModelHandle {
    async fn create(database: &Database, metadata: Metadata) -> Result<Self> {
        const ADD_MODEL: &str = r#"
        INSERT INTO model (
            name,
            num_layers,
            neurons_per_layer,
            activation_function,
            num_total_parameters,
            dataset
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6
        );
        "#;

        let metadata2 = metadata.clone();

        database
            .connection
            .call(move |connection| {
                connection.execute(
                    ADD_MODEL,
                    (
                        &metadata2.name,
                        &metadata2.layers.len(),
                        &metadata2.layers[0].num_neurons,
                        &metadata2.activation_function,
                        &metadata2.num_total_parameters,
                        &metadata2.dataset,
                    ),
                )
            })
            .await?;
        let model = ModelHandle { metadata };

        model.add_service(database, "metadata").await?;

        Ok(model)
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub async fn add_service(&self, database: &Database, service_name: &str) -> Result<()> {
        const ADD_MODEL_SERVICE: &str = r#"
        INSERT INTO model_service (
            model,
            service
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (self.name().to_owned(), service_name.to_owned());

        database
            .connection
            .call(|connection| connection.execute(ADD_MODEL_SERVICE, params))
            .await?;

        Ok(())
    }

    pub async fn add_data_object(
        &self,
        database: &Database,
        data_object_name: impl AsRef<str>,
    ) -> Result<()> {
        const ADD_DATA_OBJECT: &str = r#"
        INSERT INTO model_data_object (
            model,
            data_object
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (self.name().to_owned(), data_object_name.as_ref().to_owned());

        database
            .connection
            .call(|connection| connection.execute(ADD_DATA_OBJECT, params).map(drop))
            .await
            .context("Failed to add data object to model.")
    }

    pub async fn has_data_object(
        &self,
        database: &Database,
        data_object_name: impl AsRef<str>,
    ) -> Result<bool> {
        const CHECK_DATA_OBJECT: &str = r#"
        SELECT 
            model
        FROM model_data_object
        WHERE model = ?1 AND data_object = ?2;
        "#;

        let data_object_name = data_object_name.as_ref();

        let params = (self.name().to_owned(), data_object_name.to_owned());
        database
            .connection
            .call(|connection| connection.prepare(CHECK_DATA_OBJECT)?.exists(params))
            .await
            .with_context(|| {
                format!(
                    "Failed to check whether model '{}' has data object '{data_object_name}'",
                    self.name()
                )
            })
    }

    pub async fn get_data_object(
        &self,
        database: &Database,
        data_object_name: impl AsRef<str>,
    ) -> Result<DataObject> {
        DataObject::new(database, data_object_name.as_ref())
            .await
            .with_context(|| format!("Failed to get data object for model '{}'.", self.name()))
    }

    pub async fn add_model_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_MODEL_DATA: &str = r#"
        INSERT INTO model_data (
            model,
            data_object,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            data,
        );

        database
            .connection
            .call(|connection| connection.execute(ADD_MODEL_DATA, params).map(drop))
            .await
            .context("Failed to add model data.")
    }

    pub async fn add_layer_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
        layer_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_LAYER_DATA: &str = r#"
        INSERT INTO layer_data (
            model,
            data_object,
            layer_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4
        );
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            data,
        );

        database
            .connection
            .call(|connection| connection.execute(ADD_LAYER_DATA, params).map(drop))
            .await
            .context("Failed to add layer data.")
    }

    pub async fn add_neuron_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_NEURON_DATA: &str = r#"
        INSERT INTO neuron_data (
            model,
            data_object,
            layer_index,
            neuron_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5
        );
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            neuron_index,
            data,
        );

        database
            .connection
            .call(|connection| connection.execute(ADD_NEURON_DATA, params).map(drop))
            .await
            .context("Failed to add neuron data.")
    }

    pub async fn get_model_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
    ) -> Result<Option<Vec<u8>>> {
        const GET_MODEL_DATA: &str = r#"
        SELECT
            data
        FROM model_data
        WHERE model = ?1 AND data_object = ?2;
        "#;

        let params = (self.name().to_owned(), data_object.as_ref().to_owned());

        database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_MODEL_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get model data for model '{}' for data object '{}'.",
                    self.name(),
                    data_object.as_ref()
                )
            })
    }

    pub async fn get_layer_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
        layer_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_LAYER_DATA: &str = r#"
        SELECT
            data
        FROM layer_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3;
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
        );
        database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_LAYER_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .context("Failed to get layer data")
    }

    pub async fn get_neuron_data(
        &self,
        database: &Database,
        data_object: impl AsRef<str>,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_NEURON_DATA: &str = r#"
        SELECT
            data
        FROM neuron_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3 AND neuron_index = ?4;
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            neuron_index,
        );

        database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_NEURON_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .context("Failed to get neuron data")
    }
}
