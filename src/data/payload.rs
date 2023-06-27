use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::server::{Service, ServiceProvider};

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    data_path: PathBuf,
    services: HashMap<String, Service>,
}

impl Payload {
    pub fn initialize(path: impl AsRef<Path>) -> Self {
        let data_path = path.as_ref().to_path_buf();
        let metadata_service_provider = ServiceProvider::Metadata;
        let metadata_service = Service::new("metadata".to_string(), metadata_service_provider);
        let services = HashMap::from([("metadata".to_string(), metadata_service)]);
        Self {
            data_path,
            services,
        }
    }

    pub fn data_path(&self) -> &Path {
        self.data_path.as_path()
    }

    pub fn model_path(&self, model_name: &str) -> PathBuf {
        self.data_path.join(model_name)
    }

    pub fn add_service(&mut self, service: Service) -> Result<()> {
        if service.name() == "metadata" {
            bail!("A payload always contains a 'metadata' service. Another cannot be added.")
        }
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

    pub fn metadata_service(&self) -> &Service {
        self.service("metadata")
            .expect("All payloads must contain a 'metadata' service.")
    }
}

impl Default for Payload {
    fn default() -> Self {
        let mut result = Self::initialize("data");

        let neuroscope_service_provider = ServiceProvider::Neuroscope;
        let neuroscope_service =
            Service::new("neuroscope".to_string(), neuroscope_service_provider);
        result.add_service(neuroscope_service).unwrap();

        let neuron2graph_service_provider = ServiceProvider::Neuron2Graph;
        let neuron2graph_service =
            Service::new("neuron2graph".to_string(), neuron2graph_service_provider);
        result.add_service(neuron2graph_service).unwrap();

        let neuron2graph_service_provider = ServiceProvider::Neuron2GraphSearch;
        let neuron2graph_service = Service::new(
            "neuron2graph-search".to_string(),
            neuron2graph_service_provider,
        );
        result.add_service(neuron2graph_service).unwrap();

        result
    }
}
