use crate::data::database::Database;

mod service;
pub use service::Service;
mod service_providers;
pub use service_providers::ServiceProvider;
mod start;
pub use start::start_server;
mod response;

pub struct State {
    database: Database,
}

impl State {
    pub fn database(&self) -> &Database {
        &self.database
    }
}
