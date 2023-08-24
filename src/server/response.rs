use core::fmt;
use std::ops::Deref;

use actix_web::{body::BoxBody, get, http::header::ContentType, web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde_json::json;
use tokio::fs;

use crate::{
    data::{Database, ModelHandle, ServiceHandle},
    server::{self, State},
    Index,
};

use super::Service;

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

    pub fn html(file: String) -> Self {
        Self {
            body: file,
            content_type: ContentType::html(),
            status: StatusCode::OK,
        }
    }

    pub fn error(error: impl fmt::Debug, status: StatusCode) -> Self {
        assert!(status.is_client_error() || status.is_server_error());
        Self {
            body: format!("{error:?}"),
            content_type: ContentType::plaintext(),
            status,
        }
    }
}

impl Responder for Response {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status)
            .content_type(self.content_type)
            .append_header(("Access-Control-Allow-Origin", "*"))
            .body(self.body)
            .respond_to(req)
    }
}

async fn service_page(
    state: &State,
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service: &Service,
    page_index: Index,
) -> Result<serde_json::Value> {
    match page_index {
        Index::Model => service.model_page(state, query, model_handle).await,
        Index::Layer(layer_index) => {
            service
                .layer_page(state, query, model_handle, layer_index)
                .await
        }
        Index::Neuron(layer_index, neuron_index) => {
            service
                .neuron_page(state, query, model_handle, layer_index, neuron_index)
                .await
        }
    }
}

async fn preprocess_model(
    model_name: impl AsRef<str>,
    database: &Database,
    page_index: Index,
) -> Result<ModelHandle> {
    let model_name = model_name.as_ref();
    if let Some(model_handle) = database.model(model_name).await? {
        page_index.valid_in_model(model_handle.metadata())?;
        Ok(model_handle)
    } else {
        Err(anyhow!("Model '{model_name}' not found."))
    }
}

async fn service_value(
    state: &State,
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service_handle: &ServiceHandle,
    page_index: Index,
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

async fn response(
    state: web::Data<State>,
    query: &serde_json::Value,
    model_name: impl AsRef<str>,
    service_name: impl AsRef<str>,
    page_index: Index,
) -> Response {
    let database = state.database();

    let model_name = model_name.as_ref();
    let model_handle = match preprocess_model(model_name, database, page_index).await {
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
        let metadata_json = metadata_json.unwrap_or(serde_json::Value::Null);
        service_value(
            state.as_ref(),
            query,
            &model_handle,
            &service_handle,
            page_index,
        )
        .await
        .map(|mut service_json| {
            service_json["metadata"] = metadata_json;
            service_json
        })
    };
    match service_json {
        Ok(page) => Response::success(page),
        Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn all_response(
    state: web::Data<State>,
    query: web::Query<serde_json::Value>,
    model_name: impl AsRef<str>,
    page_index: Index,
) -> Response {
    let database = state.database();

    let model_handle = match preprocess_model(model_name, database, page_index).await {
        Ok(model_handle) => model_handle,
        Err(error) => return Response::error(error, StatusCode::NOT_FOUND),
    };

    let query = query.deref();

    let services = match ServiceHandle::all_services(database).await {
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

async fn index_data(database: &Database) -> Result<serde_json::Value> {
    let models = database.all_models().await?;
    let mut model_data: Vec<_> = Vec::with_capacity(models.len());
    for ref model_handle in models {
        let model_value = server::metadata_value(model_handle).await?;
        model_data.push(model_value);
    }
    Ok(json!({ "models": model_data }))
}

/// Gets an index over available models.
#[utoipa::path(
    operation_id = "index",
    responses(
        (status = 200, description = "Successfully retrieved the index data.", body = String, content_type = "application/json"),
        (status = "5XX", description = "Failed to retrieve the index data.", body = String) 
    )
)]
#[get("/api")]
pub async fn api_index(state: web::Data<State>) -> impl Responder {
    let database = state.database();
    match index_data(database).await {
        Ok(data) => Response::success(data),
        Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Gets the data for the specified model and service.
#[utoipa::path(
    operation_id = "model_service",
    responses(
        (status = 200, description = "Successfully retrieved data for the specified model and service.", content(
            ("application/json" = NeuroscopeModelPage),
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for the specified model and service.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for.")
    )
)]
#[get("/api/{model_name}/{service_name}")]
pub async fn model(
    state: web::Data<State>,
    indices: web::Path<(String, String)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name) = indices.into_inner();
    log::debug!("Received request for service '{service_name}' for model '{model_name}'.");
    response(state, query.deref(), model_name, service_name, Index::Model).await
}

/// Gets the data for the specified layer and service.
#[utoipa::path(
    operation_id = "layer_service",
    responses(
        (status = 200, description = "Successfully retrieved data for the specified layer and service.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for the specified layer and service.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for.")
    )
)]
#[get("/api/{model_name}/{service}/{layer_index}")]
pub async fn layer(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index) = indices.into_inner();
    log::debug!("Received request for service '{service_name}' for layer {layer_index} in model '{model_name}'.");
    response(
        state,
        query.deref(),
        model_name,
        service_name,
        Index::Layer(layer_index),
    )
    .await
}

/// Gets the data for the specified model and service.
#[utoipa::path(
    operation_id = "neuron_service",
    responses(
        (status = 200, description = "Successfully retrieved data for the specified neuron and service.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for the specified neuron and service.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for."),
        ("neuron_index" = u32, Path, description = "The index of the neuron to fetch data for.")
    )
)]
#[get("/api/{model_name}/{service}/{layer_index}/{neuron_index}")]
pub async fn neuron(
    state: web::Data<State>,
    indices: web::Path<(String, String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, service_name, layer_index, neuron_index) = indices.into_inner();
    log::debug!("Received request for service '{service_name}' for neuron l{layer_index}n{neuron_index} in model '{model_name}'.");
    response(
        state,
        query.deref(),
        model_name,
        service_name,
        Index::Neuron(layer_index, neuron_index),
    )
    .await
}

/// Gets the data for all services for the specified model.
#[utoipa::path(
    operation_id = "model_all",
    responses(
        (status = 200, description = "Successfully retrieved data for all services for the specified model.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for all services for the specified model.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for.")
    )
)]
#[get("/api/{model_name}/all")]
pub async fn all_model(
    state: web::Data<State>,
    indices: web::Path<String>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let model_name = indices.into_inner();
    log::debug!("Received request for all services for model '{model_name}'.");
    all_response(state, query, model_name, Index::Model).await
}

/// Gets the data for all services for the specified layer.
#[utoipa::path(
    operation_id = "layer_all",
    responses(
        (status = 200, description = "Successfully retrieved data for all services for the specified layer.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for all services for the specified layer.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for.")
    )
)]
#[get("/api/{model_name}/all/{layer_index}")]
pub async fn all_layer(
    state: web::Data<State>,
    indices: web::Path<(String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index) = indices.into_inner();
    log::debug!(
        "Received request for all services for layer {layer_index} in model '{model_name}'."
    );
    all_response(state, query, model_name, Index::Layer(layer_index)).await
}

/// Gets the data for all services for the specified neuron.
#[utoipa::path(
    operation_id = "neuron_all",
    responses(
        (status = 200, description = "Successfully retrieved data for all services for the specified neuron.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve data for all services for the specified neuron.", body = String) 
    ),
    params(
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for."),
        ("neuron_index" = u32, Path, description = "The index of the neuron to fetch data for.")
    )
)]
#[get("/api/{model_name}/all/{layer_index}/{neuron_index}")]
pub async fn all_neuron(
    state: web::Data<State>,
    indices: web::Path<(String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (model_name, layer_index, neuron_index) = indices.into_inner();
    log::debug!("Received request for all services for neuron l{layer_index}n{neuron_index} in model '{model_name}'.");
    all_response(
        state,
        query,
        model_name,
        Index::Neuron(layer_index, neuron_index),
    )
    .await
}

/// Gets the API documentation in JSON format.
#[utoipa::path(
    operation_id = "api_doc",
    responses(
        (status = 200, description = "Successfully retrieved API documentation.", content(
            ("application/json" = String)
        )),
        (status = "5XX", description = "Failed to retrieve API documentation.", body = String) 
    )
)]
#[get("/doc/openapi.json")]
pub async fn api_doc(state: web::Data<State>) -> impl Responder {
    log::debug!("Sending API documentation.");
    Response::success(serde_json::to_value(state.api_doc()).unwrap()) // This should always succeed.
}

async fn viz_response(file: &str) -> Response {
    log::debug!("Sending viz file '{file}.html'.");
    match fs::read_to_string(format!("frontend/{file}.html")).await {
        Ok(file) => Response::html(file),
        Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[get("/")]
pub async fn base() -> impl Responder {
    viz_response("index").await
}

#[get("/viz")]
pub async fn index_viz() -> impl Responder {
    viz_response("index").await
}

#[get("/viz/{model_name}/{service}")]
pub async fn model_viz() -> impl Responder {
    viz_response("model").await
}

#[get("/viz/{model_name}/{service}/{layer_index}")]
pub async fn layer_viz() -> impl Responder {
    viz_response("layer").await
}

#[get("/viz/{model_name}/{service}/{layer_index}/{neuron_index}")]
pub async fn neuron_viz() -> impl Responder {
    viz_response("neuron").await
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl actix_web::Responder {
    let favicon: Vec<u8> = include_bytes!("../../media/favicon.ico").to_vec();
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(favicon)
}
