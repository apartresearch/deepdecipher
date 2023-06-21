use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    path::Path,
    sync::Arc,
};

use actix_web::{
    get,
    http::header::ContentType,
    rt,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum TokenSearchType {
    Activating,
    Important,
}

impl TokenSearchType {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Activating => "activating",
            Self::Important => "important",
        }
    }

    pub fn from_str(s: &str) -> Result<Vec<Self>> {
        match s {
            "activating" => Ok(vec![Self::Activating]),
            "important" => Ok(vec![Self::Important]),
            "any" => Ok(vec![Self::Activating, Self::Important]),
            _ => bail!("Invalid token search type: '{s}'."),
        }
    }
}

impl Display for TokenSearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NeuronStore {
    activating: HashMap<String, HashSet<String>>,
    important: HashMap<String, HashSet<String>>,
}

impl NeuronStore {
    pub fn load(model: &str) -> Result<Self> {
        let neuron_store_path = Path::new("data")
            .join(model)
            .join("neuron2graph-search")
            .join("neuron_store.json");
        let neuron_store_path = neuron_store_path.as_path();
        serde_json::from_reader(
            File::open(neuron_store_path).with_context(|| {
                format!("Could not find neuron store file for model '{model}'.")
            })?,
        )
        .with_context(|| format!("Failed to parse neuron store for model '{model}'."))
    }

    pub fn get(&self, search_type: TokenSearchType, token: &str) -> Option<&HashSet<String>> {
        match search_type {
            TokenSearchType::Activating => self.activating.get(token),
            TokenSearchType::Important => self.important.get(token),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenSearch {
    token: String,
    search_types: Vec<TokenSearchType>,
}

async fn neuron2graph_search_page(
    state: &State,
    model: &str,
    query: serde_json::Value,
) -> Result<serde_json::Value> {
    let query = query["query"]
        .as_str()
        .context("Query should contain an entry 'query' with a string value.")?;
    let neuron_store = state.neuron_store(model).await?;
    let token_searches = query
        .split(',')
        .map(|token_search_string| {
            let (search_type_str, token) = token_search_string
                .split(':')
                .collect_tuple()
                .context("Token search string should be of the form 'search_type:token'.")?;
            let search_types = TokenSearchType::from_str(search_type_str)?;
            Ok(TokenSearch {
                token: token.to_string(),
                search_types,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let results = token_searches
        .into_iter()
        .map(|token_search| {
            let TokenSearch {
                token,
                search_types,
            } = token_search;
            search_types
                .into_iter()
                .flat_map(|search_type| {
                    neuron_store
                        .get(search_type, token.as_str())
                        .cloned()
                        .unwrap_or_default()
                })
                .collect::<HashSet<_>>()
        })
        .reduce(|a, b| {
            a.intersection(&b)
                .map(|str| str.to_owned())
                .collect::<HashSet<String>>()
        })
        .with_context(|| "At least one token search should be provided.")?;
    Ok(json!(results))
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
            .content_type(ContentType::json())
            .body(page.to_string()),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

#[get("/api/{model}/neuron2graph-search")]
async fn neuron2graph_search(
    state: web::Data<State>,
    model: web::Path<String>,
    web::Query(query): web::Query<serde_json::Value>,
) -> impl Responder {
    let model = model.as_str();

    match neuron2graph_search_page(state.as_ref(), model, query).await {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::json())
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

struct State {
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
                .service(neuron_2_graph)
                .service(neuron2graph_search)
                .service(all)
        })
        .bind((url, port))?
        .run(),
    )
}
