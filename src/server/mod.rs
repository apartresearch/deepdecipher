use std::path::Path;

use actix_web::{get, rt, web, App, HttpServer, Responder};

use crate::data::NeuroscopePage;

#[get("/api/solu-1l/neuroscope/{layer_index}/{neuron_index}")]
async fn index(indices: web::Path<(u32, u32)>) -> impl Responder {
    let (layer_index, neuron_index) = indices.into_inner();
    let path = Path::new("data")
        .join("solu-1l")
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}.postcard",));
    let page = NeuroscopePage::from_file(path).unwrap();
    serde_json::to_string(&page).unwrap()
}

pub fn start_server() -> std::io::Result<()> {
    rt::System::new().block_on(
        HttpServer::new(|| App::new().service(index))
            .bind(("127.0.0.1", 8080))?
            .run(),
    )
}
