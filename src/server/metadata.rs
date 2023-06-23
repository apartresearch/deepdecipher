use std::{fs, path::Path};

use actix_web::{get, http::header::ContentType, web, HttpResponse, Responder};
use anyhow::{Context, Result};
use serde_json::json;

use crate::data::ModelMetadata;

pub fn model_page(model_name: &str) -> Result<serde_json::Value> {
    let path: std::path::PathBuf = Path::new("data").join(model_name).join("metadata.json");
    let text = fs::read_to_string(path)?;
    let metadata = serde_json::from_str(&text)?;
    Ok(metadata)
}

pub fn layer_page(model_name: &str, layer_index: u32) -> Result<serde_json::Value> {
    let path = Path::new("data").join(model_name).join("metadata.json");
    let text = fs::read_to_string(path)?;
    let model_metadata: ModelMetadata = serde_json::from_str(&text)?;
    let layer_metadata = &model_metadata
        .layers
        .get(layer_index as usize)
        .context("Layer index out of bounds.")?;
    let metadata = serde_json::to_value(layer_metadata)?;
    Ok(metadata)
}

#[get("/api/{model}/metadata")]
pub async fn model(indices: web::Path<String>) -> impl Responder {
    let model_name = indices.into_inner();
    let model_name = model_name.as_str();
    let model_metadata = model_page(model_name).unwrap_or_else(|_| json!(null));

    match model_page(model_name) {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json!({"model": model_metadata, "metadata": page}).to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/metadata/{layer_index}")]
pub async fn layer(indices: web::Path<(String, u32)>) -> impl Responder {
    let (model_name, layer_index) = indices.into_inner();
    let model_name = model_name.as_str();
    let model_metadata = model_page(model_name).unwrap_or_else(|_| json!(null));

    match layer_page(model_name, layer_index) {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json!({"model": model_metadata, "metadata": page}).to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}
