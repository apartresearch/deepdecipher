use anyhow::{bail, Result};
use delegate::delegate;
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;
use crate::server::{Service, State};

#[allow(unused_variables)]
pub trait ServiceProviderTrait: Clone + Serialize + Deserialize<'static> + Send + Sync {
    fn model_page(
        &self,
        service: &Service,
        state: &State,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        bail!("No model page exists for service '{}'.", service.name());
    }
    fn layer_page(
        &self,
        service: &Service,
        state: &State,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        bail!("No layer page exists for service '{}'.", service.name());
    }
    fn neuron_page(
        &self,
        service: &Service,
        state: &State,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        bail!("No neuron page exists for service '{}'.", service.name());
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServiceProvider {
    Metadata,
}

impl ServiceProvider {
    delegate! {
        to match self {
            ServiceProvider::Metadata => Metadata,
        } {
            pub fn model_page(
                &self,
                service: &Service,
                state: &State,
                model_name: &str,
            ) -> Result<serde_json::Value>;

            pub fn layer_page(
                &self,
                service: &Service,
                state: &State,
                model_name: &str,
                layer_index: u32,
            ) -> Result<serde_json::Value>;

            pub fn neuron_page(
                &self,
                service: &Service,
                state: &State,
                model_name: &str,
                layer_index: u32,
                neuron_index: u32,
            ) -> Result<serde_json::Value>;
        }
    }
}
