use std::{
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::{
    rt::{self},
    web, Error, HttpRequest, HttpResponse,
};
use actix_ws::{Message, MessageStream, Session};
use futures_util::StreamExt as _;
use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};

const SIZE: usize = 400 * 300 * 4;

#[derive(Serialize)]
pub struct TestValue {
    pixels: Vec<u8>,
}

pub async fn connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream)?;

    let (tx, rx) = mpsc::channel::<Vec<u8>>(100);

    let tx1 = tx.clone();
    receive_message(stream, tx1);

    let tx2 = tx.clone();
    game_loop(tx2);

    send_message(session, rx);

    Ok(res)
}

fn receive_message(mut stream: MessageStream, tx1: Sender<Vec<u8>>) {
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received message: {}", text);
                }
                Ok(Message::Close(reason)) => {
                    println!("Closing due to {:?}", reason);
                    tx1.send(vec![]).await.unwrap();
                }
                _ => {}
            }
        }
    });
}

fn game_loop(tx2: mpsc::Sender<Vec<u8>>) {
    thread::spawn(move || {
        let mut x = 0;
        let mut start = SystemTime::now();
        while !tx2.is_closed() {
            x += 1;

            print_fps(&mut start, &mut x);

            let epoch = UNIX_EPOCH.elapsed().unwrap().as_millis() as usize;
            let value: TestValue = TestValue {
                pixels: (0..SIZE).map(|i| (i * epoch % 255) as u8).collect(),
            };

            let _ = futures::executor::block_on(tx2.send(value.pixels));
            // thread::sleep(Duration::from_millis(1));
        }
    });
}

fn send_message(mut session: Session, mut rx: Receiver<Vec<u8>>) {
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

fn print_fps(start: &mut SystemTime, x: &mut i32) {
    if start.elapsed().unwrap().as_nanos() >= 1_000_000_000 {
        print!("{esc}c", esc = 27 as char);
        println!("Fps: {x}");
        *x = 0;
        *start = SystemTime::now();
    }
}
