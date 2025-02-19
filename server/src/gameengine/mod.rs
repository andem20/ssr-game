mod sprite;

use std::time::SystemTime;

use sprite::TestSprite;
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

pub trait Drawable {
    fn draw_rect(
        &self,
        buffer: &mut Vec<u8>,
        dimensions: (usize, usize),
        x: i32,
        y: i32,
        width: usize,
        height: usize,
        color: [u8; 4],
    ) {
        for i in y.max(0) as usize..(y + height as i32).max(0).min(dimensions.1 as i32) as usize {
            let row = i * dimensions.0;
            let range = (row + x.max(0) as usize) * DEPTH
                ..(row + (x + width as i32).max(0).min(dimensions.0 as i32) as usize) * DEPTH;
            let mut replacement = color.repeat(range.len() / DEPTH);
            let buffer_slice = &buffer[range.clone()];

            for i in (0..replacement.len()).step_by(DEPTH) {
                let a1 = buffer_slice[i + 3] as u32;
                let a2 = replacement[i + 3] as u32;
                replacement[i] = calc_color(buffer_slice[i], replacement[i], a1, a2);
                replacement[i + 1] = calc_color(buffer_slice[i + 1], replacement[i + 1], a1, a2);
                replacement[i + 2] = calc_color(buffer_slice[i + 2], replacement[i + 2], a1, a2);
                replacement[i + 3] = 0xff;
            }

            buffer.splice(range, replacement);
        }
    }

    #[allow(dead_code)]
    fn draw_circle(
        &self,
        buffer: &mut Vec<u8>,
        dimensions: (usize, usize),
        x: i32,
        y: i32,
        radius: i32,
    ) {
        let width = dimensions.0 as i32;
        let height = dimensions.1 as i32;
        let end = (y + radius * 2).max(0).min(height);
        let range = y.max(0)..end;

        for i in range {
            let angle = f64::asin((i - y).abs_diff(radius) as f64 / radius as f64);
            let point_x = (f64::cos(angle) * radius as f64) as i32;

            let row = i * width;
            let start = (row + (x - point_x + radius).min(width).max(0)) as usize * DEPTH;
            let end = (row + (x + point_x + radius).min(width).max(0)) as usize * DEPTH;

            let filling = vec![255; (start..end).len()];

            buffer.splice(start..end, filling);
        }
    }
}

fn calc_color(c1: u8, c2: u8, a1: u32, a2: u32) -> u8 {
    return ((c1 as u32 * a1 * (0xff - a2) + c2 as u32 * a2 * 0xff) >> 16) as u8;
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
            Box::new(TestSprite::new(0, 0, 50, 20, [255, 0, 0, 127])) as Box<dyn Updatable>;

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

    fn update_user_inputs(&mut self) {
        while let Ok(input) = self.rx.try_recv() {
            self.mouse_x = input.get(0).map_or(self.mouse_x, |x| *x as usize);
            self.mouse_y = input.get(1).map_or(self.mouse_y, |y| *y as usize);
            self.keys[0] = input[2] as usize * 4;
            self.keys[1] = input[3] as usize * 4;
            self.keys[2] = input[4] as usize * 4;
            self.keys[3] = input[5] as usize * 4;
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
