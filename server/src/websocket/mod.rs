use std::time::Duration;

use actix_web::{
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
                Ok(Message::Text(_text)) => {
                    let value = TestValue {
                        pixels: vec![1, 2, 3],
                    };

                    let message = serde_json::to_string(&value).unwrap();
                    let _ = tx1.send(message).await.unwrap();
                }
                _ => {}
            }
        }
    });

    let tx2 = tx.clone();
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_micros(1_000_000));
        loop {
            interval.tick().await;
            let _ = tx2.send("test".to_owned()).await;
        }
    });

    rt::spawn(async move {
        println!("listen");
        while let Some(msg) = rx.recv().await {
            println!("Message to send: {}", msg);
            session.text(msg).await.unwrap();
        }

        println!("closed")
    });

    Ok(res)
}
