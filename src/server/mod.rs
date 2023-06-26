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

#[derive(Clone, Copy, Debug)]
enum PageIndex {
    Model,
    Layer(u32),
    Neuron(u32, u32),
}

async fn service_page(
    state: &State,
    query: web::Query<serde_json::Value>,
    service: &Service,
    model_name: &str,
    page_index: PageIndex,
) -> Result<serde_json::Value> {
    match page_index {
        PageIndex::Model => service.model_page(state, query, model_name).await,
        PageIndex::Layer(layer_index) => {
            service
                .layer_page(state, query, model_name, layer_index)
                .await
        }
        PageIndex::Neuron(layer_index, neuron_index) => {
            service
                .neuron_page(state, query, model_name, layer_index, neuron_index)
                .await
        }
    }
}

async fn response(
    state: web::Data<State>,
    query: web::Query<serde_json::Value>,
    service_name: impl AsRef<str>,
    model_name: impl AsRef<str>,
    page_index: PageIndex,
) -> impl Responder {
    let service_name = service_name.as_ref();
    let model_name = model_name.as_ref();

    if let Some(service) = state.payload().service(service_name) {
        let metadata_json = service_page(
            state.as_ref(),
            query.clone(),
            state.payload().metadata_service(),
            model_name,
            page_index,
        )
        .await;
        let service_json = if service.is_metadata() {
            metadata_json
        } else {
            let service_json =
                service_page(state.as_ref(), query, service, model_name, page_index).await;
            let metadata_json = metadata_json.unwrap_or(serde_json::Value::Null);
            service_json.map(|service_json| {
                json!({
                    "metadata": metadata_json,
                    "data": service_json
                })
            })
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

async fn all_response(
    state: web::Data<State>,
    query: web::Query<serde_json::Value>,
    model_name: impl AsRef<str>,
    page_index: PageIndex,
) -> impl Responder {
    let model_name = model_name.as_ref();

    let mut value = json!({});

    for service in state.payload().services() {
        if let Ok(page) = service_page(
            state.as_ref(),
            query.clone(),
            service,
            model_name,
            page_index,
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

#[get("/api/{model_name}/{service}")]
pub async fn model(
    state: web::Data<State>,
    indices: web::Path<(String, String)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name) = indices.into_inner();
    response(state, query, service_name, model_name, PageIndex::Model).await
}

#[get("/api/{model_name}/{service}/{layer_index}")]
pub async fn layer(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index) = indices.into_inner();
    response(
        state,
        query,
        service_name,
        model_name,
        PageIndex::Layer(layer_index),
    )
    .await
}

#[get("/api/{model_name}/{service}/{layer_index}/{neuron_index}")]
pub async fn neuron(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index, neuron_index) = indices.into_inner();
    response(
        state,
        query,
        service_name,
        model_name,
        PageIndex::Neuron(layer_index, neuron_index),
    )
    .await
}

#[get("/api/{model_name}/all")]
async fn all_model(
    state: web::Data<State>,
    indices: web::Path<String>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let model_name = indices.into_inner();
    all_response(state, query, model_name, PageIndex::Model).await
}

#[get("/api/{model_name}/all/{layer_index}")]
async fn all_layer(
    state: web::Data<State>,
    indices: web::Path<(String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index) = indices.into_inner();
    all_response(state, query, model_name, PageIndex::Layer(layer_index)).await
}

#[get("/api/{model_name}/all/{layer_index}/{neuron_index}")]
async fn all_neuron(
    state: web::Data<State>,
    indices: web::Path<(String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index, neuron_index) = indices.into_inner();
    all_response(
        state,
        query,
        model_name,
        PageIndex::Neuron(layer_index, neuron_index),
    )
    .await
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
