use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{
    cookie::time::Instant,
    rt::{self, time},
    web, Error, HttpRequest, HttpResponse,
};
use actix_ws::Message;
use futures_util::StreamExt as _;
use serde::Serialize;
use tokio::sync::mpsc;

#[derive(Serialize)]
pub struct TestValue {
    pixels: Vec<i32>,
}

pub async fn connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    let (tx, mut rx) = mpsc::channel::<String>(100);

    let tx1 = tx.clone();
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                // Ok(Message::Text(_text)) => {
                //     let value = TestValue {
                //         pixels: (0..600 * 800 * 3).map(|i| i % 255).collect(),
                //     };

                //     let message = serde_json::to_string(&value).unwrap();
                //     let _ = tx1.send(message).await.unwrap();
                // }
                _ => {}
            }
        }
    });

    let tx2 = tx.clone();
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_micros(1_000_000));
        loop {
            interval.tick().await;

            let value: TestValue = TestValue {
                pixels: (0..300 * 150 * 4)
                    .map(|i| {
                        (i * SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            % 255) as i32
                    })
                    .collect(),
            };

            let message = serde_json::to_string(&value).unwrap();
            let _ = tx2.send(message).await;
        }
    });

    rt::spawn(async move {
        println!("listen");
        while let Some(msg) = rx.recv().await {
            session.text(msg).await.unwrap();
        }

        println!("closed")
    });

    Ok(res)
}
