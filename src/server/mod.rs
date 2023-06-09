use actix_web::{get, rt, App, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    "hellooo"
}

pub fn start_server() -> std::io::Result<()> {
    rt::System::new().block_on(
        HttpServer::new(|| App::new().service(index))
            .bind(("127.0.0.1", 8080))?
            .run(),
    )
}
