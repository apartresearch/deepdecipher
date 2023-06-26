use std::path::Path;

use actix_web::web;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::{NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage},
    server::State,
};

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuroscope;

#[async_trait]
impl ServiceProviderTrait for Neuroscope {
    async fn model_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: web::Query<serde_json::Value>,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        let path = Path::new("data")
            .join(model_name)
            .join("neuroscope")
            .join("model.postcard");
        NeuroscopeModelPage::from_file(path).map(|page| json!(page))
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: web::Query<serde_json::Value>,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let path = Path::new("data")
            .join(model_name)
            .join("neuroscope")
            .join(format!("l{layer_index}.postcard",));
        NeuroscopeLayerPage::from_file(path).map(|page| json!(page))
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        _state: &State,
        _query: web::Query<serde_json::Value>,
        model_name: &str,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let path = Path::new("data")
            .join(model_name)
            .join("neuroscope")
            .join(format!("l{layer_index}n{neuron_index}.postcard",));
        NeuroscopeNeuronPage::from_file(path).map(|page| json!(page))
    }
}
