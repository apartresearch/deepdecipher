use super::{Database, Operation};

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

pub struct ServiceHandle {
    id: i64,
    name: String,
    database: Database,
}

impl ServiceHandle {
    fn create_inner(
        database: Database,
        service_name: String,
        provider_name: String,
        provider_args: Vec<u8>,
    ) -> impl Operation<Self> {
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

        let params = (service_name.clone(), provider_name, provider_args);

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
        service_name: impl AsRef<str>,
        provider: impl AsRef<str>,
        provider_args: impl AsRef<[u8]>,
    ) -> Result<Self> {
        database
            .execute(Self::create_inner(
                database.clone(),
                service_name.as_ref().to_owned(),
                provider.as_ref().to_owned(),
                provider_args.as_ref().to_vec(),
            ))
            .await
    }

    pub(super) fn new_inner(
        database: Database,
        service_name: String,
    ) -> impl Operation<Option<Self>> {
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

    pub(super) fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
}
