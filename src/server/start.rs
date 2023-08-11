use actix_web::{web, App, HttpServer};
use anyhow::{bail, Result};

use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    cli::ServerConfig,
    data::Database,
    logging,
    server::{self, response, State},
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
    let state = web::Data::new(State { database });

    let api_doc = server::api_doc();
    //let api_doc: utoipa::openapi::OpenApi =
    // serde_json::from_str(&fs::read_to_string("api.json").unwrap()).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/doc/openapi.json", api_doc.clone()))
            .service(RapiDoc::new("/doc/openapi.json").path("/rapidoc"))
            .service(Redoc::with_url("/redoc", api_doc.clone()))
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
            .service(response::favicon)
    })
    .bind((url, port))?
    .run()
    .await?;

    bail!("Server stopped unexpectedly.");
}
