use super::Database;

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

pub struct ServiceHandle {
    id: i64,
    name: String,
    database: Database,
}

impl ServiceHandle {
    pub(super) async fn create(
        database: Database,
        service_name: impl AsRef<str>,
        provider: impl AsRef<str>,
        provider_args: impl AsRef<[u8]>,
    ) -> Result<ServiceHandle> {
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

        let service_name = service_name.as_ref();
        let provider = provider.as_ref();
        let provider_args = provider_args.as_ref();

        let params = (
            service_name.to_owned(),
            provider.to_owned(),
            provider_args.to_vec(),
        );

        database
            .connection
            .call(|connection| connection.execute(ADD_SERVICE, params))
            .await?;

        let id = database.latest_id("service").await?;
        Ok(Self {
            id,
            name: service_name.to_owned(),
            database,
        })
    }

    pub(super) async fn new(database: Database, service_name: String) -> Result<Option<Self>> {
        const GET_SERVICE: &str = r#"
    SELECT
        id,
        name
    FROM service
    WHERE name = ?1;
    "#;

        let params = (service_name.clone(),);
        let service_data = database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_SERVICE)?;
                statement
                    .query_row(params, |row| Ok((row.get(0)?, row.get(1)?)))
                    .optional()
            })
            .await?;

        Ok(service_data.map(|(id, name)| ServiceHandle { id, name, database }))
    }

    pub(super) fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn delete(self) -> Result<()> {
        const DELETE_SERVICE_REFERENCES: &str = r#"
        DELETE FROM $DATABASE
        WHERE service_id = ?1;
        "#;
        const DELETE_SERVICE: &str = r#"
        DELETE FROM service
        WHERE id = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 1] = ["model_service"];

        let name = self.name();

        for table in REFERENCE_TABLES.iter() {
            let params = (self.id,);
            self.database
                .connection
                .call(move |connection| {
                    let mut statement = connection.prepare(
                        DELETE_SERVICE_REFERENCES
                            .replace("$DATABASE", table)
                            .as_str(),
                    )?;
                    statement.execute(params)?;
                    Ok(())
                })
                .await?;
        }
        let params = (self.id,);
        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(DELETE_SERVICE)?;
                statement.execute(params)?;
                Ok(())
            })
            .await
            .with_context(|| format!("Problem deleting service '{name}'."))
    }
}
