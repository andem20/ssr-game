use std::{io::Read, u16};

use actix_web::{
    rt::{self},
    web::{self},
    Error, HttpRequest, HttpResponse,
};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt as _;
use image::EncodableLayout;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::gameengine::{GameEngine, SsrGameEngine};

pub async fn connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream)?;

    let (tx, rx) = mpsc::channel::<Vec<u8>>(100);
    let (user_input_tx, user_input_rx) = mpsc::channel::<Vec<u16>>(100);

    let tx1 = tx.clone();
    handle_receive_message(stream, tx1, user_input_tx);

    let tx2 = tx.clone();
    let gameengine = SsrGameEngine::new((800, 600), tx2, user_input_rx);
    gameengine.start();

    handle_send_message(session, rx);

    Ok(res)
}

fn handle_receive_message(
    mut stream: MessageStream,
    tx1: Sender<Vec<u8>>,
    user_input_tx: Sender<Vec<u16>>,
) {
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {}", text);
                }
                Ok(Message::Binary(bytes)) => {
                    let data = bytes
                        .as_bytes()
                        .to_vec()
                        .chunks_exact(2)
                        .into_iter()
                        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                        .collect::<Vec<u16>>();

                    user_input_tx.send(data).await.unwrap();
                }
                Ok(Message::Close(reason)) => {
                    println!("Closing due to {:?}", reason);
                    tx1.send(vec![]).await.unwrap();
                }
                Err(e) => {
                    println!("An error occured: {}", e)
                }
                _ => println!("Operation not supported"),
            }
        }
    });
}

fn handle_send_message(mut session: Session, mut rx: Receiver<Vec<u8>>) {
    rt::spawn(async move {
        loop {
            if let Some(msg) = rx.recv().await {
                if msg.len() == 0 {
                    println!("Closed");
                    rx.close();
                    break;
                } else {
                    let _ = session.binary(msg).await;
                }
            }
        }
    });
}
