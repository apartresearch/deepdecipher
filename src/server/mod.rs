use std::{fs, path::Path};

use actix_web::{
    get, http::header::ContentType, rt, web, App, HttpResponse, HttpServer, Responder,
};

use crate::data::NeuroscopePage;

#[get("/api/{model}/neuroscope/{layer_index}/{neuron_index}")]
async fn neuroscope(indices: web::Path<(String, u32, u32)>) -> impl Responder {
    let (model, layer_index, neuron_index) = indices.into_inner();
    let model = model.as_str();
    let path = Path::new("data")
        .join(model)
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}.postcard",));
    match NeuroscopePage::from_file(path) {
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
    let path = Path::new("data")
        .join(model)
        .join("neuron2graph")
        .join(format!("layer_{layer_index}",))
        .join(format!("{layer_index}_{neuron_index}"))
        .join("graph");
    println!("{:?}", path);
    match fs::read_to_string(path) {
        Ok(page) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(page),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

pub fn start_server() -> std::io::Result<()> {
    rt::System::new().block_on(
        HttpServer::new(|| App::new().service(neuroscope).service(neuron_2_graph))
            .bind(("127.0.0.1", 8080))?
            .run(),
    )
}
