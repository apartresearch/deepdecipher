use std::ops::Deref;

use actix_web::{
    get,
    http::header::ContentType,
    rt,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;

use serde_json::json;

use crate::data::{database::Database, ModelHandle};

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
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service: &Service,
    page_index: PageIndex,
) -> Result<serde_json::Value> {
    match page_index {
        PageIndex::Model => service.model_page(state, query, model_handle).await,
        PageIndex::Layer(layer_index) => {
            service
                .layer_page(state, query, model_handle, layer_index)
                .await
        }
        PageIndex::Neuron(layer_index, neuron_index) => {
            service
                .neuron_page(state, query, model_handle, layer_index, neuron_index)
                .await
        }
    }
}

async fn response(
    state: web::Data<State>,
    query: &serde_json::Value,
    model_name: impl AsRef<str>,
    service_name: impl AsRef<str>,
    page_index: PageIndex,
) -> impl Responder {
    let model_name = model_name.as_ref();
    let model_handle = match state.database().model(model_name).await {
        Ok(Some(model_handle)) => model_handle,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!("Model '{model_name}' not found.",))
        }
        Err(error) => return HttpResponse::InternalServerError().body(format!("{error:?}")),
    };
    let service_name = service_name.as_ref();
    let service_handle = match state.database().service(service_name).await {
        Ok(Some(service_handle)) => service_handle,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!("Service '{service_name}' not found.",))
        }
        Err(error) => return HttpResponse::InternalServerError().body(format!("{error:?}")),
    };
    let service = match service_handle.service().await {
        Ok(service) => service,
        Err(error) => return HttpResponse::InternalServerError().body(format!("{error:?}")),
    };

    let metadata_json = service_page(
        state.as_ref(),
        query,
        &model_handle,
        &Service::metadata(),
        page_index,
    )
    .await;
    let service_json = if service.is_metadata() {
        metadata_json
    } else {
        let service_json =
            service_page(state.as_ref(), query, &model_handle, &service, page_index).await;
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
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error:?}")),
    }
}

async fn all_response(
    state: web::Data<State>,
    query: web::Query<serde_json::Value>,
    model_name: impl AsRef<str>,
    page_index: PageIndex,
) -> impl Responder {
    let model_name = model_name.as_ref();
    let model_handle = match state.database().model(model_name).await {
        Ok(Some(model_handle)) => model_handle,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!("Model '{model_name}' not found.",))
        }
        Err(error) => return HttpResponse::InternalServerError().body(format!("{error:?}")),
    };

    let query = query.deref();

    let mut value = json!({});

    let services = match model_handle.services().await {
        Ok(services) => services,
        Err(error) => return HttpResponse::InternalServerError().body(format!("{error:?}")),
    };

    for service in services {
        if let Ok(page) =
            service_page(state.as_ref(), query, &model_handle, &service, page_index).await
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
    response(
        state,
        query.deref(),
        model_name,
        service_name,
        PageIndex::Model,
    )
    .await
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
        query.deref(),
        model_name,
        service_name,
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
        query.deref(),
        model_name,
        service_name,
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
    database: Database,
}

impl State {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }
}

pub fn start_server(database: Database) -> std::io::Result<()> {
    let url = "127.0.0.1";
    let port = 8080;
    println!("Serving neuronav on http://{url}:{port}/");
    let state = web::Data::new(State::new(database));
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
