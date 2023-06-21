use std::{fs, path::Path};

use actix_web::{
    get,
    http::header::ContentType,
    rt,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::{Context, Result};
use serde_json::json;

use crate::data::NeuroscopePage;

async fn neuroscope_page(
    model: &str,
    layer_index: u32,
    neuron_index: u32,
) -> Result<serde_json::Value> {
    let path = Path::new("data")
        .join(model)
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}.postcard",));
    NeuroscopePage::from_file(path).map(|page| json!(page))
}

async fn neuron2graph_page(
    model: &str,
    layer_index: u32,
    neuron_index: u32,
) -> Result<serde_json::Value> {
    let path = Path::new("data")
        .join(model)
        .join("neuron2graph")
        .join(format!("layer_{layer_index}",))
        .join(format!("{layer_index}_{neuron_index}"))
        .join("graph");
    fs::read_to_string(path).map(|page| json!(page)).with_context(|| format!("Failed to read neuron2graph page for neuron {neuron_index} in layer {layer_index} of model '{model}'."))
}

#[get("/api/{model}/neuroscope/{layer_index}/{neuron_index}")]
async fn neuroscope(indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();

    match neuroscope_page(model, layer_index, neuron_index).await {
        Ok(page) => HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(&page)
                .expect("Failed to serialize page to JSON. This should always be possible."),
        ),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/neuron2graph/{layer_index}/{neuron_index}")]
async fn neuron_2_graph(indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();

    match neuron2graph_page(model, layer_index, neuron_index).await {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(page.to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/all/{layer_index}/{neuron_index}")]
async fn all(indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();

    let neuroscope_page = neuroscope_page(model, layer_index, neuron_index).await;
    let neuron2graph_page = neuron2graph_page(model, layer_index, neuron_index).await;

    match (neuroscope_page, neuron2graph_page) {
        (Ok(neuroscope_page), Ok(neuron2graph_page)) => {
            HttpResponse::Ok().content_type(ContentType::json()).body(
                json!({
                    "neuroscope": neuroscope_page,
                    "neuron2graph": neuron2graph_page,
                })
                .to_string(),
            )
        }
        (Ok(neuroscope_page), _) => HttpResponse::Ok().content_type(ContentType::json()).body(
            json!({
                "neuroscope": neuroscope_page,
            })
            .to_string(),
        ),
        (_, Ok(neuron2graph_page)) => HttpResponse::Ok().content_type(ContentType::json()).body(
            json!({
                "neuron2graph": neuron2graph_page,
            })
            .to_string(),
        ),
        _ => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json!({}).to_string()),
    }
}

pub fn start_server() -> std::io::Result<()> {
    let url = "127.0.0.1";
    let port = 8080;
    println!("Serving neuronav on http://{url}:{port}/");
    rt::System::new().block_on(
        HttpServer::new(|| {
            App::new()
                .service(neuroscope)
                .service(neuron_2_graph)
                .service(all)
        })
        .bind((url, port))?
        .run(),
    )
}
