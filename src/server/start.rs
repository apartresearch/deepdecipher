use actix_web::{web, App, HttpServer};
use anyhow::{bail, Result};

use crate::{
    cli::ServerConfig,
    data::Database,
    logging,
    server::{response, State},
};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    logging::log_init(&config);

    let database_path = config.database_path();
    let database = if database_path.exists() {
        log::info!("Opening database at {:?}.", database_path);
        Database::open(database_path).await?
    } else {
        log::error!("Database not found at {database_path:?}.");
        bail!("Database not found at {database_path:?}.");
    };
    let url = "127.0.0.1";
    let port = config.port();
    log::info!("Serving DeepDecipher on http://{url}:{port}/");
    let state = web::Data::new(State { database });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(response::api_index)
            .service(response::all_model)
            .service(response::all_layer)
            .service(response::all_neuron)
            .service(response::model)
            .service(response::layer)
            .service(response::neuron)
            .service(actix_files::Files::new("/js", "./frontend/js"))
            .service(actix_files::Files::new("/css", "./frontend/css"))
            .service(response::base)
            .service(response::index_viz)
            .service(response::model_viz)
            .service(response::layer_viz)
            .service(response::neuron_viz)
    })
    .bind((url, port))?
    .run()
    .await?;

    bail!("Server stopped unexpectedly.");
}
