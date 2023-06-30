use std::path::Path;

use anyhow::{bail, Context, Result};
use rusqlite::{Params, ToSql};
use tokio_rusqlite::Connection;

use crate::data::LayerMetadata;

use super::Metadata;

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

const TABLES: [&str; 7] = [
    MODEL_TABLE,
    SERVICE_TABLE,
    MODEL_SERVICE_TABLE,
    DATA_OBJECT_TABLE,
    MODEL_DATA_TABLE,
    LAYER_DATA_TABLE,
    NEURON_DATA_TABLE,
];

pub struct Database {
    connection: Connection,
}

impl Database {
    pub async fn initialize(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().exists() {
            bail!("Database already exists at {:?}", path.as_ref())
        }

        let database = Connection::open(path).await?;

        let mut database = Database {
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

    pub async fn add_model(&mut self, metadata: Metadata) -> Result<ModelHandle> {
        ModelHandle::create(self, metadata).await
    }

    pub async fn model(&self, model_name: String) -> Result<ModelHandle> {
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

                let row = rows.next()?.unwrap();

                let num_layers: u32 = row.get(1)?;
                let neurons_per_layer = row.get(2)?;
                let layers = vec![
                    LayerMetadata {
                        num_neurons: neurons_per_layer
                    };
                    num_layers as usize
                ];

                Ok(Metadata {
                    name: row.get(0)?,
                    layers,
                    activation_function: row.get(3)?,
                    num_total_neurons: num_layers * neurons_per_layer,
                    num_total_parameters: row.get(4)?,
                    dataset: row.get(5)?,
                })
            })
            .await?;

        Ok(ModelHandle { metadata })
    }

    pub async fn add_service(
        &mut self,
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

    pub async fn get_model_data(
        &self,
        model_name: impl AsRef<str>,
        data_object: impl AsRef<str>,
    ) -> Result<Vec<u8>> {
        const GET_MODEL_DATA: &str = r#"
        SELECT
            data
        FROM model_data
        WHERE model = ?1 AND data_object = ?2;
        "#;

        let params = (
            model_name.as_ref().to_owned(),
            data_object.as_ref().to_owned(),
        );

        self.connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_MODEL_DATA)?;
                statement.query_row(params, |row| row.get(0))
            })
            .await
            .context("Failed to get model data.")
    }

    pub async fn get_layer_data(
        &self,
        model_name: impl AsRef<str>,
        data_object: impl AsRef<str>,
        layer_index: u32,
    ) -> Result<Vec<u8>> {
        const GET_LAYER_DATA: &str = r#"
        SELECT
            data
        FROM layer_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3;
        "#;

        let params = (
            model_name.as_ref().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
        );
        self.connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_LAYER_DATA)?;
                statement.query_row(params, |row| row.get(0))
            })
            .await
            .context("Failed to get layer data")
    }

    pub async fn get_neuron_data(
        &self,
        model_name: impl AsRef<str>,
        data_object: impl AsRef<str>,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Vec<u8>> {
        const GET_NEURON_DATA: &str = r#"
        SELECT
            data
        FROM neuron_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3 AND neuron_index = ?4;
        "#;

        let params = (
            model_name.as_ref().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            neuron_index,
        );

        self.connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_NEURON_DATA)?;
                statement.query_row(params, |row| row.get(0))
            })
            .await
            .context("Failed to get neuron data")
    }
}

pub struct ModelHandle {
    metadata: Metadata,
}

impl ModelHandle {
    async fn create(database: &mut Database, metadata: Metadata) -> Result<Self> {
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

    pub async fn add_service(&self, database: &mut Database, service_name: &str) -> Result<()> {
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
}
