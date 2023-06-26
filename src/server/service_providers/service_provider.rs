use std::{future::Future, pin::Pin};

use actix_web::web;
use anyhow::{bail, Result};
use async_trait::async_trait;
use delegate::delegate;
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;
use crate::server::{Service, State};

#[allow(unused_variables)]
#[async_trait]
pub trait ServiceProviderTrait: Clone + Serialize + Deserialize<'static> + Send + Sync {
    async fn model_page(
        &self,
        service: &Service,
        state: &State,
        query: web::Query<serde_json::Value>,
        model_name: &str,
    ) -> Result<serde_json::Value> {
        bail!("No model page exists for service '{}'.", service.name());
    }
    async fn layer_page(
        &self,
        service: &Service,
        state: &State,
        query: web::Query<serde_json::Value>,
        model_name: &str,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        bail!("No layer page exists for service '{}'.", service.name());
    }
    async fn neuron_page(
        &self,
        service: &Service,
        state: &State,
        query: web::Query<serde_json::Value>,
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
    pub fn is_metadata(&self) -> bool {
        matches!(self, ServiceProvider::Metadata)
    }

    delegate! {
        to match self {
            ServiceProvider::Metadata => Metadata,
        } {
            pub fn model_page<'a>(
                &'a self,
                service: &'a Service,
                state: &'a State,
                query: web::Query<serde_json::Value>,
                model_name: &'a str,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a>>;

            pub fn layer_page<'a>(
                &'a self,
                service: &'a Service,
                state: &'a State,
                query: web::Query<serde_json::Value>,
                model_name: &'a str,
                layer_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a >>;

            pub fn neuron_page<'a>(
                &'a self,
                service: &'a Service,
                state: &'a State,
                query: web::Query<serde_json::Value>,
                model_name: &'a str,
                layer_index: u32,
                neuron_index: u32,
            ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + 'a >>;
        }
    }
}
