use crate::server::{Service, ServiceProvider};

use super::{DataObjectHandle, Database, Operation};

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

#[derive(Clone)]
pub struct ServiceHandle {
    id: i64,
    name: String,
    database: Database,
}

impl ServiceHandle {
    fn create_inner(
        database: Database,
        service_name: String,
        provider_bytes: Vec<u8>,
    ) -> impl Operation<Self> {
        const ADD_SERVICE: &str = r#"
        INSERT INTO service (
            name,
            provider
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (service_name.clone(), provider_bytes);

        |transaction| {
            let id = transaction
                .prepare(ADD_SERVICE)
                .with_context(|| format!("Failed to add service '{}'.", service_name.as_str()))?
                .insert(params)?;
            Ok(Self {
                id,
                name: service_name,
                database,
            })
        }
    }

    pub(super) async fn create(
        mut database: Database,
        Service { name, provider }: Service,
    ) -> Result<Self> {
        database
            .execute(Self::create_inner(
                database.clone(),
                name,
                provider.to_binary()?,
            ))
            .await
    }

    pub fn new_inner(database: Database, service_name: String) -> impl Operation<Option<Self>> {
        const GET_SERVICE: &str = r#"
        SELECT
            id,
            name
        FROM service
        WHERE name = ?1;
        "#;

        let params = (service_name,);

        |transaction| {
            let service_data = transaction
                .prepare(GET_SERVICE)?
                .query_row(params, |row| Ok((row.get(0)?, row.get(1)?)))
                .optional()?;
            let service = service_data.map(|(id, name)| ServiceHandle { id, name, database });
            Ok(service)
        }
    }

    pub(super) async fn new(mut database: Database, service_name: String) -> Result<Option<Self>> {
        database
            .execute(Self::new_inner(database.clone(), service_name))
            .await
    }

    pub async fn all_services(database: &Database) -> Result<impl Iterator<Item = ServiceHandle>> {
        const ALL_SERVICES: &str = r#"
            SELECT id, name
            FROM service
        "#;

        let database2 = database.clone();
        let services = database
            .connection
            .call(move |connection| {
                let services = connection
                    .prepare(ALL_SERVICES)?
                    .query_map((), |row| Ok((row.get(0)?, row.get(1)?)))?
                    .map(|row| {
                        row.map(|(id, name)| ServiceHandle {
                            id,
                            name,
                            database: database2.clone(),
                        })
                    })
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok(services)
            })
            .await?;
        Ok(services.into_iter())
    }

    pub(super) fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn required_data_objects(&self) -> Result<Vec<DataObjectHandle>> {
        self.service()
            .await?
            .required_data_objects(&self.database)
            .await
    }

    fn delete_inner(&self) -> impl Operation<()> {
        const DELETE_SERVICE_REFERENCES: &str = r#"
        DELETE FROM $TABLE
        WHERE service_id = ?1;
        "#;
        const DELETE_SERVICE: &str = r#"
        DELETE FROM service
        WHERE id = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 1] = ["model_service"];

        let params = (self.id,);
        move |transaction| {
            for table in REFERENCE_TABLES.iter() {
                let mut statement = transaction
                    .prepare(DELETE_SERVICE_REFERENCES.replace("$TABLE", table).as_str())?;
                statement.execute(params)?;
            }
            transaction.prepare(DELETE_SERVICE)?.execute(params)?;
            Ok(())
        }
    }

    pub async fn delete(mut self) -> Result<()> {
        let name = self.name().to_owned();
        self.database
            .execute(self.delete_inner())
            .await
            .with_context(|| format!("Problem deleting service '{name}'."))
    }

    pub async fn service(&self) -> Result<Service> {
        const GET_SERVICE: &str = r#"
        SELECT 
            provider
        FROM service 
        WHERE id = ?1;"#;

        let params = (self.id(),);

        let provider_bytes: Vec<u8> = self
            .database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_SERVICE)?;
                statement
                    .query_row(params, |row| row.get::<_, Vec<u8>>(0))
                    .optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get service provider data for service '{}'.",
                    self.name(),
                )
            })?
            .with_context(|| {
                format!(
                    "Row for service '{}' with name '{}' does not exist.",
                    self.id(),
                    self.name()
                )
            })?;

        let service_provider = ServiceProvider::from_binary(provider_bytes)?;
        Ok(Service::new(self.name.to_owned(), service_provider))
    }
}
