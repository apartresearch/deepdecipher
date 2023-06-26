use std::collections::HashMap;

use crate::server::Service;

pub struct Payload {
    services: HashMap<String, Service>,
}

impl Payload {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, service: Service) {
        self.services.insert(service.name().to_string(), service);
    }

    pub fn services(&self) -> &HashMap<String, Service> {
        &self.services
    }
}
