use core::f64;

use actix_web::{web, App, HttpServer};
use server::websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/connect", web::get().to(websocket::connect)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

// fn main() {
//     let radius = 10.0;

//     for i in 0..10 {
//         let angle = (i as f64 / 10.0) * f64::consts::PI;
//         let x = f64::cos(angle);
//         let y = f64::sin(angle);
//         println!("({}, {})", (x * radius) as i32, (y * radius) as i32);
//     }
// }

// x x o x x
// x o x o x
// x o x o x
// x x o x x
// x x x x x
