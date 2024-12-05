use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc::Sender;

use crate::config;

pub trait GameEngine {
    fn start(self);
    fn stop(&self);
    fn tick(&self);
    fn render(&self);
}

pub struct UserInput {
    keys: [u8; 4],
}

pub trait Tickable: Send {
    fn tick(&self);
}

pub struct SsrGameEngine {
    dimensions: (usize, usize),
    buffer_size: usize,
    user_input: UserInput,
    tx: Sender<Vec<u8>>,
    tickables: Vec<Box<dyn Tickable>>,
    offset: usize,
}

impl UserInput {
    pub fn new() -> Self {
        Self { keys: [0; 4] }
    }

    pub fn keys(&self) -> [u8; 4] {
        self.keys
    }
}

impl SsrGameEngine {
    pub fn new(dimensions: (usize, usize), tx: Sender<Vec<u8>>) -> Self {
        Self {
            dimensions,
            buffer_size: dimensions.0 * dimensions.1 * 4, // RGBA
            user_input: UserInput::new(),
            tx,
            tickables: vec![],
            offset: 0,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    pub fn user_input(&self) -> &UserInput {
        &self.user_input
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl GameEngine for SsrGameEngine {
    fn start(mut self) {
        // Listen for userinput
        // Create thread
        std::thread::spawn(move || {
            let this = &mut self;
            let mut x = 0;
            let mut start = SystemTime::now();
            while !this.tx.is_closed() {
                x += 1;

                print_fps(&mut start, &mut x);

                this.render();

                this.offset = (this.offset + 2) % (this.dimensions().0 * 4);

                // thread::sleep(Duration::from_millis(1));
            }
        });
    }

    fn stop(&self) {
        todo!()
    }

    fn render(&self) {
        let mut buffer = vec![0_u8; self.buffer_size()];

        for i in 100..120 {
            buffer.splice(
                (i * 4 * self.dimensions().0) + self.offset
                    ..(i * 4 * self.dimensions().0) + 200 + self.offset,
                [255; 200],
            );
        }

        let _ = futures::executor::block_on(self.tx.send(buffer));
    }

    fn tick(&self) {
        todo!()
    }
}

fn print_fps(start: &mut SystemTime, x: &mut i32) {
    if start.elapsed().unwrap().as_nanos() >= 1_000_000_000 {
        // print!("{esc}c", esc = 27 as char);
        println!("Thread: {:?}, Fps: {}", std::thread::current().id(), x);
        *x = 0;
        *start = SystemTime::now();
    }
}
