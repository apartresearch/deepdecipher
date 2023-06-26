use std::{collections::HashMap, sync::Arc};

use actix_web::{
    get,
    http::header::ContentType,
    rt,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;

use serde_json::json;
use tokio::sync::Mutex;

use crate::data::{NeuronStore, Payload};

mod service;
pub use service::Service;
mod service_providers;
pub use service_providers::ServiceProvider;

#[get("/api/{model_name}/{service}")]
pub async fn model(
    state: web::Data<State>,
    indices: web::Path<(String, String)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name) = indices.into_inner();
    let model_name = model_name.as_str();
    let service_name = service_name.as_str();

    if let Some(service) = state.payload().service(service_name) {
        let service_json = if service.name() == "metadata" {
            service.model_page(state.as_ref(), query, model_name).await
        } else {
            match service
                .model_page(state.as_ref(), query.clone(), model_name)
                .await
            {
                Ok(service_json) => {
                    let metadata_json = ServiceProvider::Metadata
                        .model_page("metadata", state.as_ref(), query, model_name)
                        .await
                        .unwrap_or(serde_json::Value::Null);

                    Ok(json!({
                        "metadata": metadata_json,
                        "data": service_json
                    }))
                }
                Err(error) => Err(error),
            }
        };
        match service_json {
            Ok(page) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(page.to_string()),
            Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
        }
    } else {
        HttpResponse::NotFound().body(format!("Service '{service_name}' not found.",))
    }
}

#[get("/api/{model_name}/{service}/{layer_index}")]
pub async fn layer(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index) = indices.into_inner();
    let model_name = model_name.as_str();
    let service_name = service_name.as_str();

    if let Some(service) = state.payload().service(service_name) {
        let service_json = if service.name() == "metadata" {
            service
                .layer_page(state.as_ref(), query, model_name, layer_index)
                .await
        } else {
            match service
                .layer_page(state.as_ref(), query.clone(), model_name, layer_index)
                .await
            {
                Ok(service_json) => {
                    let metadata_json = ServiceProvider::Metadata
                        .layer_page("metadata", state.as_ref(), query, model_name, layer_index)
                        .await
                        .unwrap_or(serde_json::Value::Null);

                    Ok(json!({
                        "metadata": metadata_json,
                        "data": service_json
                    }))
                }
                Err(error) => Err(error),
            }
        };
        match service_json {
            Ok(page) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(page.to_string()),
            Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
        }
    } else {
        HttpResponse::NotFound().body(format!("Service '{service_name}' not found.",))
    }
}

#[get("/api/{model_name}/{service}/{layer_index}/{neuron_index}")]
pub async fn neuron(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index, neuron_index) = indices.into_inner();
    let model_name = model_name.as_str();
    let service_name = service_name.as_str();

    if let Some(service) = state.payload().service(service_name) {
        let service_json = if service.name() == "metadata" {
            service
                .neuron_page(state.as_ref(), query, model_name, layer_index, neuron_index)
                .await
        } else {
            match service
                .neuron_page(
                    state.as_ref(),
                    query.clone(),
                    model_name,
                    layer_index,
                    neuron_index,
                )
                .await
            {
                Ok(service_json) => {
                    let metadata_json = ServiceProvider::Metadata
                        .neuron_page(
                            "metadata",
                            state.as_ref(),
                            query,
                            model_name,
                            layer_index,
                            neuron_index,
                        )
                        .await
                        .unwrap_or(serde_json::Value::Null);

                    Ok(json!({
                        "metadata": metadata_json,
                        "data": service_json
                    }))
                }
                Err(error) => Err(error),
            }
        };
        match service_json {
            Ok(page) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(page.to_string()),
            Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
        }
    } else {
        HttpResponse::NotFound().body(format!("Service '{service_name}' not found.",))
    }
}

#[get("/api/{model_name}/all")]
async fn all_model(
    state: web::Data<State>,
    indices: web::Path<String>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let model_name = indices.into_inner();
    let model_name = model_name.as_str();

    let mut value = json!({});

    for service in state.payload().services() {
        if let Ok(page) = service
            .model_page(state.as_ref(), query.clone(), model_name)
            .await
        {
            value[service.name()] = page;
        }
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(value.to_string())
}

#[get("/api/{model_name}/all/{layer_index}")]
async fn all_layer(
    state: web::Data<State>,
    indices: web::Path<(String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index) = indices.into_inner();
    let model_name = model_name.as_str();

    let mut value = json!({});

    for service in state.payload().services() {
        if let Ok(page) = service
            .layer_page(state.as_ref(), query.clone(), model_name, layer_index)
            .await
        {
            value[service.name()] = page;
        }
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(value.to_string())
}

#[get("/api/{model_name}/all/{layer_index}/{neuron_index}")]
async fn all_neuron(
    state: web::Data<State>,
    indices: web::Path<(String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index, neuron_index) = indices.into_inner();
    let model_name = model_name.as_str();

    let mut value = json!({});

    for service in state.payload().services() {
        if let Ok(page) = service
            .neuron_page(
                state.as_ref(),
                query.clone(),
                model_name,
                layer_index,
                neuron_index,
            )
            .await
        {
            value[service.name()] = page;
        }
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(value.to_string())
}

pub struct State {
    neuron_stores: Arc<Mutex<HashMap<String, NeuronStore>>>,
    payload: Payload,
}

impl State {
    pub fn new(payload: Payload) -> Self {
        Self {
            neuron_stores: Arc::new(Mutex::new(HashMap::new())),
            payload,
        }
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub async fn neuron_store(&self, model_name: &str) -> Result<NeuronStore> {
        let mut neuron_stores = self.neuron_stores.lock().await;
        if !neuron_stores.contains_key(model_name) {
            log::info!("Neuron store doesn't exist for model '{model_name}', loading from disk");
            neuron_stores.insert(model_name.to_string(), NeuronStore::load(model_name)?);
        }
        assert!(neuron_stores.contains_key(model_name));
        Ok(neuron_stores.get(model_name).unwrap().clone())
    }
}

impl Default for State {
    fn default() -> Self {
        let payload = Payload::default();
        Self::new(payload)
    }
}

pub fn start_server() -> std::io::Result<()> {
    let url = "127.0.0.1";
    let port = 8080;
    println!("Serving neuronav on http://{url}:{port}/");
    let state = web::Data::new(State::default());
    rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .service(all_model)
                .service(all_layer)
                .service(all_neuron)
                .service(model)
                .service(layer)
                .service(neuron)
        })
        .bind((url, port))?
        .run(),
    )
}
