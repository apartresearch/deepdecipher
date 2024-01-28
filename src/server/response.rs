use core::fmt;
use std::ops::Deref;

use actix_web::{body::BoxBody, get, http::header::ContentType, web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde_json::json;

use super::{RequestType, Service};
use crate::{
    data::{data_objects::MetadataObject, Database, ModelHandle, ServiceHandle},
    server::State,
    Index,
};

pub enum Body {
    Json(serde_json::Value),
    Binary(Vec<u8>),
    String(String),
}

impl Body {
    pub fn content_type(&self) -> ContentType {
        match self {
            Body::Json(_) => ContentType::json(),
            Body::Binary(_) => ContentType::octet_stream(),
            Body::String(_) => ContentType::plaintext(),
        }
    }
}

impl From<serde_json::Value> for Body {
    fn from(value: serde_json::Value) -> Self {
        Body::Json(value)
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body::Binary(value)
    }
}

impl From<Body> for BoxBody {
    fn from(value: Body) -> Self {
        match value {
            Body::Json(value) => BoxBody::new(value.to_string()),
            Body::Binary(value) => BoxBody::new(value),
            Body::String(value) => BoxBody::new(value),
        }
    }
}

struct Response {
    body: Body,
    status: StatusCode,
}

impl Response {
    pub fn success(body: impl Into<Body>) -> Self {
        Self {
            body: body.into(),
            status: StatusCode::OK,
        }
    }

    pub fn error(error: impl fmt::Debug, status: StatusCode) -> Self {
        assert!(status.is_client_error() || status.is_server_error());
        Self {
            body: Body::String(format!("{error:?}")),
            status,
        }
    }
}

impl Responder for Response {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status)
            .content_type(self.body.content_type())
            .append_header(("Access-Control-Allow-Origin", "*"))
            .body(BoxBody::from(self.body))
            .respond_to(req)
    }
}

async fn service_json(
    state: &State,
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service: &Service,
    page_index: Index,
) -> Result<serde_json::Value> {
    match page_index {
        Index::Model => service.model_json(state, query, model_handle).await,
        Index::Layer(layer_index) => {
            service
                .layer_json(state, query, model_handle, layer_index)
                .await
        }
        Index::Neuron(layer_index, neuron_index) => {
            service
                .neuron_json(state, query, model_handle, layer_index, neuron_index)
                .await
        }
    }
}

async fn service_binary(
    state: &State,
    query: &serde_json::Value,
    model_handle: &ModelHandle,
    service: &Service,
    page_index: Index,
) -> Result<Vec<u8>> {
    match page_index {
        Index::Model => service.model_binary(state, query, model_handle).await,
        Index::Layer(layer_index) => {
            service
                .layer_binary(state, query, model_handle, layer_index)
                .await
        }
        Index::Neuron(layer_index, neuron_index) => {
            service
                .neuron_binary(state, query, model_handle, layer_index, neuron_index)
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
    let missing_data_types = model_handle.missing_data_types(service_handle).await?;
    let service = service_handle.service().await?;

    if missing_data_types.is_empty() {
        let page = service_json(state, query, model_handle, &service, page_index).await?;
        Ok(json!({ "data": page }))
    } else {
        Ok(json!({ "missing_data_types": missing_data_types }))
    }
}

async fn response(
    state: web::Data<State>,
    query: &serde_json::Value,
    request_type: RequestType,
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

    match request_type {
        RequestType::Json => {
            let metadata_json = service_json(
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
                Ok(page) => Response::success(Body::Json(page)),
                Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        RequestType::Binary => {
            let data =
                service_binary(state.as_ref(), query, &model_handle, &service, page_index).await;
            match data {
                Ok(data) => Response::success(Body::Binary(data)),
                Err(error) => Response::error(error, StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
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
        let model_value = json!(MetadataObject::new(model_handle).await?);
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
        ("request_type" = String, Path, description = "The type of request to make. Must be either 'api' or 'bin'."),
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for.")
    )
)]
#[get("/{request_type}/{model_name}/{service_name}")]
pub async fn model(
    state: web::Data<State>,
    indices: web::Path<(String, String, String)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (request_type_string, model_name, service_name) = indices.into_inner();
    let request_type = match RequestType::from_path_string(&request_type_string) {
        Ok(request_type) => request_type,
        Err(error) => return Response::error(error, StatusCode::BAD_REQUEST),
    };
    log::debug!(
        "Received {request_type_string} request for service '{service_name}' for model \
         '{model_name}'."
    );
    response(
        state,
        query.deref(),
        request_type,
        model_name,
        service_name,
        Index::Model,
    )
    .await
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
        ("request_type" = String, Path, description = "The type of request to make. Must be either 'api' or 'bin'."),
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for.")
    )
)]
#[get("/api/{model_name}/{service}/{layer_index}")]
pub async fn layer(
    state: web::Data<State>,
    indices: web::Path<(String, String, String, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (request_type_string, model_name, service_name, layer_index) = indices.into_inner();
    let request_type = match RequestType::from_path_string(&request_type_string) {
        Ok(request_type) => request_type,
        Err(error) => return Response::error(error, StatusCode::BAD_REQUEST),
    };
    log::debug!(
        "Received {request_type_string} request for service '{service_name}' for layer \
         {layer_index} in model '{model_name}'."
    );
    response(
        state,
        query.deref(),
        request_type,
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
        ("request_type" = String, Path, description = "The type of request to make. Must be either 'api' or 'bin'."),
        ("model_name" = String, Path, description = "The name of the model to fetch data for."),
        ("service_name" = String, Path, description = "The name of the service to fetch data for."),
        ("layer_index" = u32, Path, description = "The index of the layer to fetch data for."),
        ("neuron_index" = u32, Path, description = "The index of the neuron to fetch data for.")
    )
)]
#[get("/{request_type}/{model_name}/{service}/{layer_index}/{neuron_index}")]
pub async fn neuron(
    state: web::Data<State>,
    indices: web::Path<(String, String, String, u32, u32)>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let (request_type_string, model_name, service_name, layer_index, neuron_index) =
        indices.into_inner();
    let request_type = match RequestType::from_path_string(&request_type_string) {
        Ok(request_type) => request_type,
        Err(error) => return Response::error(error, StatusCode::BAD_REQUEST),
    };
    log::debug!(
        "Received {request_type_string} request for service '{service_name}' for neuron \
         l{layer_index}n{neuron_index} in model '{model_name}'."
    );
    response(
        state,
        query.deref(),
        request_type,
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
    log::debug!(
        "Received request for all services for neuron l{layer_index}n{neuron_index} in model \
         '{model_name}'."
    );
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
