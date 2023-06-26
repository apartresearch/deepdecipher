use anyhow::{bail, Result};

use super::{service::Service, State};

#[allow(unused_variables)]
pub trait ServiceProvider {
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
