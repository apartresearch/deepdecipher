use anyhow::{bail, Result};

use crate::data::database::Database;

mod service;
pub use service::Service;
mod service_providers;
pub use service_providers::ServiceProvider;
mod start;
pub use start::start_server;
mod api_doc;
pub mod response;
pub use api_doc::api_doc;

pub struct State {
    api_doc: utoipa::openapi::OpenApi,
    database: Database,
}

impl State {
    pub fn new(database: Database) -> Result<Self> {
        let api_doc = api_doc();
        Ok(Self { api_doc, database })
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn api_doc(&self) -> &utoipa::openapi::OpenApi {
        &self.api_doc
    }
}

#[derive(Clone, Copy)]
pub enum RequestType {
    Json,
    Binary,
}

impl RequestType {
    pub fn from_path_string(s: impl AsRef<str>) -> Result<Self> {
        match s.as_ref() {
            "api" => Ok(Self::Json),
            "bin" => Ok(Self::Binary),
            s => bail!("Invalid request type '{s}'."),
        }
    }
}
