use macroquad::prelude::*;
use crate::core::assets::GameAssets;

pub struct MatrixEntity {
    x: f32,
    y: f32,
    speed: f32,
    symbol: char,
    switch_timer: f32,
}

pub struct MatrixStream {
    entities: Vec<MatrixEntity>,
}

pub struct MatrixBackground {
    streams: Vec<MatrixStream>,
}

impl MatrixBackground {
    pub fn new() -> Self {
        let mut streams = Vec::new();
        let spacing = 20.0;
        let stream_count = (screen_width() / spacing) as usize;

        for i in 0..stream_count {
            let initial_y = rand::gen_range(-screen_height(), screen_height());
            streams.push(MatrixStream::new(i as f32 * spacing, initial_y));
        }

        Self { streams }
    }

    pub fn update(&mut self, dt: f32) {
        for stream in &mut self.streams {
            stream.update(dt);
        }
    }

    pub fn draw(&self, assets: &GameAssets) {
        for stream in &self.streams {
            stream.draw(assets);
        }
    }
}

impl MatrixStream {
    fn new(x: f32, start_y: f32) -> Self {
        let mut entities = Vec::new();
        let count = rand::gen_range(8, 25);
        let speed = rand::gen_range(70.0, 200.0);

        for i in 0..count {
            entities.push(MatrixEntity {
                x,
                y: start_y - (i as f32 * 22.0),
                speed,
                symbol: ' ',
                switch_timer: rand::gen_range(0.0, 0.5),
            });
        }
        Self { entities }
    }

    fn update(&mut self, dt: f32) {
        for e in &mut self.entities {
            e.y += e.speed * dt;
            
            e.switch_timer -= dt;
            if e.switch_timer <= 0.0 {
                e.symbol = if rand::gen_range(0, 5) > 0 {
                    rand::gen_range(33, 126) as u8 as char
                } else {
                    if rand::gen_range(0, 2) == 0 { '0' } else { '1' }
                };
                e.switch_timer = rand::gen_range(0.1, 1.0);
            }
        }

        if !self.entities.is_empty() && self.entities[0].y > screen_height() + 100.0 {
            let new_speed = rand::gen_range(80.0, 220.0);
            let reset_y = rand::gen_range(-400.0, -50.0);
            
            for (i, e) in self.entities.iter_mut().enumerate() {
                e.speed = new_speed;
                e.y = reset_y - (i as f32 * 22.0);
            }
        }
    }

    fn draw(&self, assets: &GameAssets) {
        for (i, e) in self.entities.iter().enumerate() {
            let alpha = 1.0 - (i as f32 / self.entities.len() as f32);
            let color = Color::new(0.0, 0.8, 0.2, alpha * 0.5); 

            draw_text_ex(
                &e.symbol.to_string(),
                e.x,
                e.y,
                TextParams {
                    font_size: 16,
                    font: Some(&assets.font),
                    color,
                    ..Default::default()
                },
            );
        }
    }
}