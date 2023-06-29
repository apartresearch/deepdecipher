use std::path::Path;

use anyhow::{bail, Result};
use rusqlite::Connection;

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

const MODEL_DATA_TABLE: &str = r#"
CREATE TABLE model_data (
    model                   TEXT NOT NULL,
    data_name               TEXT NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_name),
    FOREIGN KEY(model) REFERENCES model(name)
  ) STRICT;
"#;

const LAYER_DATA_TABLE: &str = r#"
CREATE TABLE layer_data (
    model                   TEXT NOT NULL,
    data_name               TEXT NOT NULL,
    layer_index             INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_name, layer_index),
    FOREIGN KEY(model) REFERENCES model(name),
    CHECK (layer_index >= 0)
  ) STRICT;
"#;

const NEURON_DATA_TABLE: &str = r#"
CREATE TABLE neuron_data (
    model                   TEXT NOT NULL,
    data_name               TEXT NOT NULL,
    layer_index             INTEGER NOT NULL,
    neuron_index            INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model, data_name, layer_index, neuron_index),
    FOREIGN KEY(model) REFERENCES model(name),
    CHECK (layer_index >= 0 AND neuron_index >= 0)
  ) STRICT;
"#;

const TABLES: [&str; 6] = [
    MODEL_TABLE,
    SERVICE_TABLE,
    MODEL_SERVICE_TABLE,
    MODEL_DATA_TABLE,
    LAYER_DATA_TABLE,
    NEURON_DATA_TABLE,
];

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn initialize(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().exists() {
            bail!("Database already exists at {:?}", path.as_ref())
        }

        let database = Connection::open(path)?;

        for table in TABLES.iter() {
            database.execute(table, ())?;
        }

        let mut database = Database {
            connection: database,
        };
        database.add_service("metadata", "metadata", [])?;

        Ok(database)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if !path.as_ref().exists() {
            bail!("Database does not exist at {:?}", path.as_ref())
        }

        let database = Connection::open(path)?;

        Ok(Database {
            connection: database,
        })
    }

    pub fn add_model(&mut self, metadata: Metadata) -> Result<ModelHandle> {
        ModelHandle::create(self, metadata)
    }

    pub fn model(&self, model_name: &str) -> Result<ModelHandle> {
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

        let mut statement = self.connection.prepare(GET_MODEL)?;
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

        let metadata = Metadata {
            name: row.get(0)?,
            layers,
            activation_function: row.get(3)?,
            num_total_neurons: num_layers * neurons_per_layer,
            num_total_parameters: row.get(4)?,
            dataset: row.get(5)?,
        };

        Ok(ModelHandle { metadata })
    }

    pub fn add_service(
        &mut self,
        service_name: &str,
        provider: &str,
        provider_args: impl AsRef<[u8]>,
    ) -> Result<()> {
        const ADD_TABLE: &str = r#"
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

        self.connection
            .execute(ADD_TABLE, (service_name, provider, provider_args.as_ref()))?;

        Ok(())
    }
}

pub struct ModelHandle {
    metadata: Metadata,
}

impl ModelHandle {
    fn create(database: &mut Database, metadata: Metadata) -> Result<Self> {
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

        database.connection.execute(
            ADD_MODEL,
            (
                &metadata.name,
                &metadata.layers.len(),
                &metadata.layers[0].num_neurons,
                &metadata.activation_function,
                &metadata.num_total_parameters,
                &metadata.dataset,
            ),
        )?;
        let model = ModelHandle { metadata };

        model.add_service(database, "metadata")?;

        Ok(model)
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn add_service(&self, database: &mut Database, service_name: &str) -> Result<()> {
        const ADD_MODEL_SERVICE: &str = r#"
        INSERT INTO model_service (
            model,
            service
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        database
            .connection
            .execute(ADD_MODEL_SERVICE, (self.name(), service_name))?;

        Ok(())
    }
}
