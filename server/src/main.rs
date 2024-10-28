use actix_web::{web, App, HttpServer};
use server::websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/connect", web::get().to(websocket::connect)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
