use std::time::SystemTime;

use tokio::sync::mpsc::{Receiver, Sender};

const DEPTH: usize = 4;
const TICKS_PR_SECOND: u128 = 60;
const NANOS: u128 = 1_000_000_000;
const TICK_DURATION: u128 = NANOS / TICKS_PR_SECOND;

pub trait GameEngine {
    fn start(self);
    fn stop(&self);
    fn update(&mut self);
    fn render(&mut self);
}

pub trait Updatable: Send {
    fn update(&mut self, inputs: [usize; 4]);
    fn render(&self, buffer: &mut Vec<u8>, dimensions: (usize, usize));
}

pub struct TestSprite {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: [u8; 4],
}

impl TestSprite {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }
}

impl Updatable for TestSprite {
    fn update(&mut self, inputs: [usize; 4]) {
        self.x -= inputs[0];
        self.x += inputs[2];
        self.y -= inputs[1];
        self.y += inputs[3];
    }

    fn render(&self, mut buffer: &mut Vec<u8>, dimensions: (usize, usize)) {
        draw_rect(
            &mut buffer,
            dimensions,
            self.x,
            self.y,
            self.width,
            self.height,
        );
    }
}

pub struct SsrGameEngine {
    dimensions: (usize, usize),
    buffer_size: usize,
    keys: [usize; 4],
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u16>>,
    sprites: Vec<Box<dyn Updatable>>,
    mouse_x: usize,
    mouse_y: usize,
}

impl SsrGameEngine {
    pub fn new(dimensions: (usize, usize), tx: Sender<Vec<u8>>, rx: Receiver<Vec<u16>>) -> Self {
        let test_sprite =
            Box::new(TestSprite::new(0, 0, 50, 20, [255, 0, 0, 255])) as Box<dyn Updatable>;

        Self {
            dimensions,
            buffer_size: dimensions.0 * dimensions.1 * DEPTH,
            keys: [0; 4],
            tx,
            rx,
            sprites: vec![test_sprite],
            mouse_x: 0,
            mouse_y: 0,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    #[allow(dead_code)]
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

    fn update_user_inputs(&mut self) {
        while let Ok(input) = self.rx.try_recv() {
            self.mouse_x = input.get(0).map_or(self.mouse_x, |x| *x as usize);
            self.mouse_y = input.get(1).map_or(self.mouse_y, |y| *y as usize);
            self.keys[0] = input[2] as usize;
            self.keys[1] = input[3] as usize;
            self.keys[2] = input[4] as usize;
            self.keys[3] = input[5] as usize;
        }
    }
}

#[allow(dead_code)]
fn draw_rect(
    buffer: &mut Vec<u8>,
    dimensions: (usize, usize),
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    for i in y..y + height {
        buffer.splice(
            ((i * dimensions.0) + x) * DEPTH..((i * dimensions.0) + x + width) * DEPTH,
            vec![124; width * DEPTH],
        );
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

                this.update();
                this.render();

                std::thread::sleep(std::time::Duration::from_nanos(
                    (TICK_DURATION - s.elapsed().as_nanos()) as u64,
                ));
            }
        });
    }

    fn stop(&self) {
        todo!()
    }

    fn render(&mut self) {
        let mut buffer = vec![0_u8; self.buffer_size()];

        self.sprites
            .iter()
            .for_each(|s| s.render(&mut buffer, self.dimensions()));

        self.draw_circle(&mut buffer, self.mouse_x, self.mouse_y, 100);

        let _ = futures::executor::block_on(self.tx.send(buffer));
    }

    fn update(&mut self) {
        self.update_user_inputs();
        self.sprites.iter_mut().for_each(|s| s.update(self.keys));
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
