use actix_web::{
    middleware::{self, TrailingSlash},
    web, App, HttpServer,
};
use anyhow::{bail, Result};

use utoipa_redoc::{Redoc, Servable};

use crate::{
    cli::ServerConfig,
    data::Database,
    logging,
    server::{response, State},
};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    logging::log_init_config(&config);

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
    let state = web::Data::new(State::new(database)?);

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .service(Redoc::with_url_and_config(
                "/doc",
                state.api_doc().clone(),
                || {
                    serde_json::from_str::<serde_json::Value>(include_str!(
                        "../../redoc_config.json"
                    ))
                    .unwrap()
                },
            ))
            .service(response::api_index)
            .service(response::all_model)
            .service(response::all_layer)
            .service(response::all_neuron)
            .service(response::model)
            .service(response::layer)
            .service(response::neuron)
            .service(response::api_doc)
            .service(actix_files::Files::new("/js", "./frontend/js"))
            .service(actix_files::Files::new("/css", "./frontend/css"))
            .service(response::base)
            .service(response::index_viz)
            .service(response::model_viz)
            .service(response::layer_viz)
            .service(response::neuron_viz)
    });
    if let Some(num_workers) = config.num_workers() {
        server = server.workers(num_workers);
    }
    server.bind((url, port))?.run().await?;

    bail!("Server stopped unexpectedly.");
}
