use std::{collections::HashMap, path::Path, sync::Arc};

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

use crate::data::NeuroscopePage;
pub mod neuron2graph;
use neuron2graph::NeuronStore;

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

#[get("/api/{model}/all/{layer_index}/{neuron_index}")]
async fn all(state: web::Data<State>, indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();

    let neuroscope_page = neuroscope_page(model, layer_index, neuron_index).await;
    let neuron2graph_page =
        neuron2graph::neuron2graph_page(state.as_ref(), model, layer_index, neuron_index).await;

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

pub struct State {
    neuron_stores: Arc<Mutex<HashMap<String, NeuronStore>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            neuron_stores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn neuron_store(&self, model: &str) -> Result<NeuronStore> {
        let mut neuron_stores = self.neuron_stores.lock().await;
        if !neuron_stores.contains_key(model) {
            log::info!("Neuron store doesn't exist for model '{model}', loading from disk");
            neuron_stores.insert(model.to_string(), NeuronStore::load(model)?);
        }
        assert!(neuron_stores.contains_key(model));
        Ok(neuron_stores.get(model).unwrap().clone())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub fn start_server() -> std::io::Result<()> {
    let url = "127.0.0.1";
    let port = 8080;
    println!("Serving neuronav on http://{url}:{port}/");
    let state = web::Data::new(State::new());
    rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .service(neuroscope)
                .service(neuron2graph::neuron_2_graph)
                .service(neuron2graph::neuron2graph_search)
                .service(all)
        })
        .bind((url, port))?
        .run(),
    )
}
