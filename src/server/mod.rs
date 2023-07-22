use std::{fmt::Debug, ops::Deref};

use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    rt,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::{anyhow, Result};

use serde_json::json;

use crate::data::{database::Database, Metadata, ModelHandle, ServiceHandle};

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

impl PageIndex {
    pub fn in_model(self, metadata: &Metadata) -> Result<()> {
        let model_name = metadata.name.as_str();
        let num_layers = metadata.num_layers;
        let layer_size = metadata.layer_size;

        match self {
            PageIndex::Layer(layer_index) | PageIndex::Neuron(layer_index, _) if layer_index >= num_layers => Err(anyhow!(
                "Layer index is {layer_index} but model '{model_name}' only has {num_layers} layers."
            )),
            PageIndex::Neuron(_, neuron_index) if neuron_index >= layer_size => Err(anyhow!(
                "Neuron index is {neuron_index} but model '{model_name}' only has {layer_size} neurons per layer."
            )),
            _ => Ok(())
        }
    }
}

struct Response {
    body: String,
    content_type: ContentType,
    status: StatusCode,
}

impl Response {
    pub fn success(body: serde_json::Value) -> Self {
        Self {
            body: body.to_string(),
            content_type: ContentType::json(),
            status: StatusCode::OK,
        }
    }

    pub fn error(error: impl Debug, status: StatusCode) -> Self {
        assert!(status.is_client_error() || status.is_server_error());
        Self {
            body: format!("{error:?}"),
            content_type: ContentType::plaintext(),
            status,
        }
    }

    pub fn finalize(self) -> impl Responder {
        HttpResponse::build(self.status)
            .append_header(("Access-Control-Allow-Origin", "*"))
            .content_type(self.content_type)
            .body(self.body)
    }
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

async fn preprocess_model(
    model_name: impl AsRef<str>,
    database: &Database,
    page_index: PageIndex,
) -> Result<ModelHandle> {
    let model_name = model_name.as_ref();
    if let Some(model_handle) = database.model(model_name).await? {
        page_index.in_model(model_handle.metadata())?;
        Ok(model_handle)
    } else {
        Err(anyhow!("Model '{model_name}' not found."))
    }
}

async fn response(
    state: web::Data<State>,
    query: &serde_json::Value,
    model_name: impl AsRef<str>,
    service_name: impl AsRef<str>,
    page_index: PageIndex,
) -> Response {
    let database = state.database();

    let model_name = model_name.as_ref();
    let model_handle = match preprocess_model(model_name, &database, page_index).await {
        Ok(model_handle) => model_handle,
        Err(error) => return Response::error(error, StatusCode::NOT_FOUND),
    };

    let service_name = service_name.as_ref();
    let service_handle = match database.service(service_name).await {
        Ok(Some(service_handle)) => service_handle,
        Ok(None) => {
            return Response::error(
                anyhow!("Service '{service_name}' not found.",),
                StatusCode::NOT_FOUND,
            )
        }
        Err(error) => return Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    };
    let service = match service_handle.service().await {
        Ok(service) => service,
        Err(error) => return Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
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
        let missing_data_objects = match model_handle.missing_data_objects(&service_handle).await {
            Ok(missing_data_objects) => missing_data_objects,
            Err(error) => return Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
        };
        let metadata_json = metadata_json.unwrap_or(serde_json::Value::Null);
        if missing_data_objects.is_empty() {
            let service_json =
                service_page(state.as_ref(), query, &model_handle, &service, page_index).await;
            service_json.map(|service_json| {
                json!({
                    "metadata": metadata_json,
                    "data": service_json
                })
            })
        } else {
            return Response::error(
                anyhow!("Service '{service_name}' unavailable for model '{model_name}' due to missing data objects: {missing_data_objects:?}"),
                StatusCode::NOT_FOUND,
            );
        }
    };
    match service_json {
        Ok(page) => Response::success(page),
        Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn service_value(
    state: &State,
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service_handle: &ServiceHandle,
    page_index: PageIndex,
) -> Result<serde_json::Value> {
    let missing_data_objects = model_handle.missing_data_objects(service_handle).await?;
    let service = service_handle.service().await?;

    if missing_data_objects.is_empty() {
        let page = service_page(state, query, model_handle, &service, page_index).await?;
        Ok(json!({ "data": page }))
    } else {
        Ok(json!({ "missing_data_objects": missing_data_objects }))
    }
}

async fn all_response(
    state: web::Data<State>,
    query: web::Query<serde_json::Value>,
    model_name: impl AsRef<str>,
    page_index: PageIndex,
) -> Response {
    let database = state.database();

    let model_handle = match preprocess_model(model_name, &database, page_index).await {
        Ok(model_handle) => model_handle,
        Err(error) => return Response::error(error, StatusCode::NOT_FOUND),
    };

    let query = query.deref();

    let services = match ServiceHandle::all_services(&database).await {
        Ok(services) => services,
        Err(error) => return Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut value = json!({});

    for service_handle in services {
        match service_value(
            state.as_ref(),
            query,
            &model_handle,
            &service_handle,
            page_index,
        )
        .await
        {
            Ok(page) => value[service_handle.name()] = page,
            Err(error) => value[service_handle.name()] = json!({ "error": format!("{error:?}") }),
        }
    }

    Response::success(value)
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
    .finalize()
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
    .finalize()
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
    .finalize()
}

#[get("/api/{model_name}/all")]
async fn all_model(
    state: web::Data<State>,
    indices: web::Path<String>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let model_name = indices.into_inner();
    all_response(state, query, model_name, PageIndex::Model)
        .await
        .finalize()
}

#[get("/api/{model_name}/all/{layer_index}")]
async fn all_layer(
    state: web::Data<State>,
    indices: web::Path<(String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index) = indices.into_inner();
    all_response(state, query, model_name, PageIndex::Layer(layer_index))
        .await
        .finalize()
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
    .finalize()
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
