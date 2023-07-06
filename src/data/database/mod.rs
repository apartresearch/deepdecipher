use std::path::Path;

use anyhow::{bail, Context, Result};

use rusqlite::Transaction;
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

mod table_definitions;
use table_definitions::TABLES;

pub trait Operation<R>: FnOnce(&mut Transaction) -> Result<R> + 'static + Send
where
    R: 'static + Send,
{
    fn call(self, transaction: &mut Transaction) -> Result<R>;
}

impl<T, R> Operation<R> for T
where
    T: FnOnce(&mut Transaction) -> Result<R> + 'static + Send,
    R: 'static + Send,
{
    fn call(self, transaction: &mut Transaction) -> Result<R> {
        self(transaction)
    }
}

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

    async fn execute<R, F>(&mut self, f: F) -> Result<R>
    where
        F: Operation<R>,
        R: Send + 'static,
    {
        self.connection
            .call(|connection| {
                let mut transaction = connection.transaction()?;
                match f(&mut transaction) {
                    Ok(result) => {
                        transaction.commit()?;
                        Ok(Ok(result))
                    }
                    Err(error) => Ok(Err(error)),
                }
            })
            .await?
    }

    pub async fn add_model(&self, metadata: Metadata) -> Result<ModelHandle> {
        ModelHandle::create(self.clone(), metadata).await
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
