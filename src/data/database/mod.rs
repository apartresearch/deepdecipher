use std::path::Path;

use anyhow::{bail, Context, Result};

use tokio_rusqlite::Connection;

use self::data_types::ModelDataObject;

use super::{data_types::DataType, Metadata};

mod model_handle;
pub use model_handle::ModelHandle;
mod data_object_handle;
pub use data_object_handle::DataObjectHandle;
pub mod data_types;
mod service_handle;
use service_handle::ServiceHandle;

const MODEL_TABLE: &str = r#"
CREATE TABLE model (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
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
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    provider                TEXT NOT NULL,
    provider_args           BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_SERVICE_TABLE: &str = r#"
CREATE TABLE model_service (
    model_id                   INTEGER NOT NULL,
    service_id                 INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(service_id) REFERENCES service(id)
  ) STRICT;
"#;

const DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE data_object (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    type                    TEXT NOT NULL,
    type_args               BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE model_data_object (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
)
"#;

const MODEL_DATA_TABLE: &str = r#"
CREATE TABLE model_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
  ) STRICT;
"#;

const LAYER_DATA_TABLE: &str = r#"
CREATE TABLE layer_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    layer_index             INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id, layer_index),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
    CHECK (layer_index >= 0)
  ) STRICT;
"#;

const NEURON_DATA_TABLE: &str = r#"
CREATE TABLE neuron_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    layer_index             INTEGER NOT NULL,
    neuron_index            INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id, layer_index, neuron_index),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
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

    async fn latest_id(&self, table: &str) -> Result<i64> {
        const LATEST_ID: &str = r#"
        SELECT seq
        FROM sqlite_sequence
        WHERE name = ?1;
        "#;

        let params = (table.to_owned(),);
        self.connection
            .call(|connection| {
                connection
                    .prepare(LATEST_ID)?
                    .query_row(params, |row| row.get(0))
            })
            .await
            .with_context(|| format!("Failed to get latest id for table '{table}'."))
    }

    pub async fn add_model(&self, metadata: Metadata) -> Result<ModelHandle> {
        ModelHandle::create(self, metadata).await
    }

    pub async fn model(&self, model_name: impl AsRef<str>) -> Result<Option<ModelHandle>> {
        ModelHandle::new(self.clone(), model_name.as_ref().to_owned()).await
    }

    pub async fn add_service(
        &self,
        service_name: impl AsRef<str>,
        provider: impl AsRef<str>,
        provider_args: impl AsRef<[u8]>,
    ) -> Result<ServiceHandle> {
        ServiceHandle::create(self.clone(), service_name, provider, provider_args).await
    }

    pub async fn service(&self, service_name: impl AsRef<str>) -> Result<Option<ServiceHandle>> {
        ServiceHandle::new(self.clone(), service_name.as_ref().to_owned()).await
    }

    pub async fn add_data_object(
        &self,
        data_object_name: impl AsRef<str>,
        data_type: DataType,
    ) -> Result<DataObjectHandle> {
        DataObjectHandle::create(
            self.clone(),
            data_object_name.as_ref().to_owned(),
            data_type,
        )
        .await
    }

    pub async fn data_object(
        &self,
        data_object_name: impl AsRef<str>,
    ) -> Result<Option<DataObjectHandle>> {
        DataObjectHandle::new(self.clone(), data_object_name.as_ref()).await
    }

    pub async fn model_data_object<D>(
        &self,
        model: &ModelHandle,
        data_object: &DataObjectHandle,
    ) -> Result<D>
    where
        D: ModelDataObject,
    {
        const HAS_MODEL_DATA_OBJECT: &str = r#"
        SELECT model FROM model_data_object WHERE model = ?1 AND data_object = ?2;
        "#;

        let model_name = model.name();
        let data_object_name = data_object.name();

        let params = (model_name.to_owned(), data_object_name.to_owned());

        if !self.connection
            .call(|connection| connection.prepare(HAS_MODEL_DATA_OBJECT)?.exists(params))
            .await.with_context(|| format!("Failed to check whether model '{model_name}' has data object '{data_object_name}'"))? {
            bail!("Model '{model_name}' does not have data object '{data_object_name}'.");
            }
        let data_object = self
            .data_object(data_object_name)
            .await?
            .context("No data object of type '{data_object_name}' found.")?;
        let result = D::new(model, data_object.data_type().clone()).await?;
        if let Some(result) = result {
            Ok(result)
        } else {
            let output_data_type = D::data_type();
            let output_data_type = output_data_type.as_ref();
            bail!("Data object '{data_object_name}' is not of type '{output_data_type}'.");
        }
    }
}
