use crate::gameengine::{Drawable, Updatable};

pub struct TestSprite {
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    color: [u8; 4],
}

impl TestSprite {
    pub fn new(x: i32, y: i32, width: usize, height: usize, color: [u8; 4]) -> Self {
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
        self.x -= inputs[0] as i32;
        self.x += inputs[2] as i32;
        self.y -= inputs[1] as i32;
        self.y += inputs[3] as i32;
    }

    fn render(&self, mut buffer: &mut Vec<u8>, dimensions: (usize, usize)) {
        // self.draw_rect(
        //     &mut buffer,
        //     dimensions,
        //     self.x,
        //     self.y,
        //     self.width,
        //     self.height,
        //     self.color,
        // );

        // self.draw_circle(&mut buffer, dimensions, self.x, self.y, 100);
        self.draw_rect(
            &mut buffer,
            dimensions,
            self.x,
            self.y,
            100,
            100,
            self.color,
        );
    }
}

impl Drawable for TestSprite {}
