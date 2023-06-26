use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::server::{Service, ServiceProvider, State};

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    services: HashMap<String, Service>,
}

impl Payload {
    pub fn add_service(&mut self, service: Service) -> Result<()> {
        if self.services.contains_key(service.name()) {
            bail!("A service named '{}' already exists.", service.name());
        }
        self.services.insert(service.name().to_string(), service);
        Ok(())
    }

    pub fn services(&self) -> impl Iterator<Item = &Service> + '_ {
        self.services.values()
    }

    pub fn service<S: AsRef<str>>(&self, key: S) -> Option<&Service> {
        self.services.get(key.as_ref())
    }
}

impl Default for Payload {
    fn default() -> Self {
        let mut result = Self {
            services: HashMap::new(),
        };

        let metadata_service_provider = ServiceProvider::Metadata;
        let metadata_service = Service::new("metadata".to_string(), metadata_service_provider);
        result.add_service(metadata_service).unwrap();

        result
    }
}
