use std::time::SystemTime;

use tokio::sync::mpsc::{Receiver, Sender};

const DEPTH: usize = 4;
const TICKS_PR_SECOND: u128 = 60;
const NANOS: u128 = 1_000_000_000;
const TICK_DURATION: u128 = NANOS / TICKS_PR_SECOND;

const HEIGHT: usize = 20;
const WIDTH: usize = 50;

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
    rx: Receiver<Vec<u16>>,
    tickables: Vec<Box<dyn Tickable>>,
    offset: usize,
    x: usize,
    y: usize,
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
    pub fn new(dimensions: (usize, usize), tx: Sender<Vec<u8>>, rx: Receiver<Vec<u16>>) -> Self {
        Self {
            dimensions,
            buffer_size: dimensions.0 * dimensions.1 * DEPTH,
            user_input: UserInput::new(),
            tx,
            rx,
            tickables: vec![],
            offset: 0,
            x: 0,
            y: 0,
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

    fn draw_circle(&self, buffer: &mut Vec<u8>, x: usize, y: usize, radius: usize) {
        for i in 0..radius * 2 {
            let angle = f64::asin(i.abs_diff(radius) as f64 / radius as f64);
            let point_x = (f64::cos(angle) * radius as f64) as usize;

            let filling = vec![255; point_x * 2 * DEPTH];

            let position = ((y + i) * self.dimensions().0) + x;
            let start = (position - point_x) * DEPTH;
            let end = (position + point_x) * DEPTH;

            buffer.splice(start..end, filling);
        }
    }

    fn draw_rect(&self, buffer: &mut Vec<u8>, x: usize, y: usize, width: usize, height: usize) {
        for i in y..y + height {
            buffer.splice(
                ((i * self.dimensions().0) + x) * DEPTH
                    ..((i * self.dimensions().0) + x + width) * DEPTH,
                vec![255; width * DEPTH],
            );
        }
    }
}

impl GameEngine for SsrGameEngine {
    fn start(mut self) {
        std::thread::spawn(move || {
            let this = &mut self;
            let mut x = 0;
            let mut start = SystemTime::now();

            while !this.tx.is_closed() {
                let s = std::time::Instant::now();
                x += 1;

                print_fps(&mut start, &mut x);

                while let Ok(input) = this.rx.try_recv() {
                    if let Some(x) = input.get(0) {
                        this.x = (&this.dimensions.0 - WIDTH).min(*x as usize);
                    }

                    if let Some(y) = input.get(1) {
                        this.y = (&this.dimensions.1 - HEIGHT).min(*y as usize);
                    }

                    // println!("Received position: {}, {}", this.x, this.y);
                }

                this.render();

                this.offset = (this.offset + 2) % (this.dimensions().0 * DEPTH);

                std::thread::sleep(std::time::Duration::from_nanos(
                    (TICK_DURATION - s.elapsed().as_nanos()) as u64,
                ));
            }
        });
    }

    fn stop(&self) {
        todo!()
    }

    fn render(&self) {
        let mut buffer = vec![0_u8; self.buffer_size()];

        self.draw_rect(&mut buffer, 10, 10, WIDTH, HEIGHT);

        self.draw_circle(&mut buffer, self.x, self.y, 100);

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
