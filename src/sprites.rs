// Sprites.rs
use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use std::time::{Duration, Instant};

pub struct AnimatedSprite {
    pub pos: Vector2,
    pub frames: Vec<Color>, 
    pub current_frame: usize,
    pub frame_duration: Duration,
    pub last_frame_time: Instant,
    pub size: u32,
}

impl AnimatedSprite {
    pub fn new(pos: Vector2, size: u32) -> Self {
        let frames = vec![
            Color::new(100, 200, 100, 255), 
            Color::new(150, 255, 150, 255), 
            Color::new(200, 255, 200, 255), 
            Color::new(150, 255, 150, 255), 
        ];

        AnimatedSprite {
            pos,
            frames,
            current_frame: 0,
            frame_duration: Duration::from_millis(200), 
            last_frame_time: Instant::now(),
            size,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_frame_time) >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_frame_time = now;
        }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer, _block_size: usize) {
        let color = self.frames[self.current_frame];
        framebuffer.set_current_color(color);

        let x = (self.pos.x) as usize;
        let y = (self.pos.y) as usize;

        for dx in 0..self.size {
            for dy in 0..self.size {
                let px = x + dx as usize;
                let py = y + dy as usize;
                
                if px < framebuffer.width as usize && py < framebuffer.height as usize {
                    framebuffer.set_pixel(px as u32, py as u32);
                }
            }
        }
    }
}