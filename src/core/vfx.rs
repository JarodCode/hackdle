use macroquad::prelude::*;
use std::f32::consts::PI;

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
}

pub struct Laser {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct VfxManager {
    pub particles: Vec<Particle>,
    pub lasers: Vec<Laser>,
    pub shake_intensity: f32,
    pub shake_duration: f32,
    pub timer: f32,
}

impl VfxManager {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            lasers: Vec::new(),
            shake_intensity: 0.0,
            shake_duration: 0.0,
            timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;

        // Update particles
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.lifetime -= dt;
            // Add a little drag
            p.velocity *= 0.95;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        // Update lasers
        for l in &mut self.lasers {
            l.lifetime -= dt;
        }
        self.lasers.retain(|l| l.lifetime > 0.0);

        // Update screen shake
        if self.shake_duration > 0.0 {
            self.shake_duration -= dt;
            // Decay intensity over time
            self.shake_intensity *= 0.9;
        } else {
            self.shake_intensity = 0.0;
        }
    }

    pub fn draw(&self, offset: Vec2) {
        // Draw particles
        for p in &self.particles {
            let alpha = p.lifetime / p.max_lifetime;
            let mut c = p.color;
            c.a = alpha;
            draw_rectangle(p.position.x - p.size / 2.0 + offset.x, p.position.y - p.size / 2.0 + offset.y, p.size, p.size, c);
        }

        // Draw lasers
        for l in &self.lasers {
            let alpha = l.lifetime / l.max_lifetime;
            let mut c = l.color;
            c.a = alpha;
            let thickness = alpha * 4.0;
            draw_line(l.start.x + offset.x, l.start.y + offset.y, l.end.x + offset.x, l.end.y + offset.y, thickness, c);
            
            // Add a glow around the laser
            let mut glow = c;
            glow.a = alpha * 0.3;
            draw_line(l.start.x + offset.x, l.start.y + offset.y, l.end.x + offset.x, l.end.y + offset.y, thickness * 3.0, glow);
        }
    }

    pub fn spawn_explosion(&mut self, pos: Vec2, count: usize, color: Color) {
        for _ in 0..count {
            let angle = rand::gen_range(0.0, PI * 2.0);
            let speed = rand::gen_range(50.0, 300.0);
            let lifetime = rand::gen_range(0.2, 0.6);
            
            self.particles.push(Particle {
                position: pos,
                velocity: vec2(angle.cos() * speed, angle.sin() * speed),
                color,
                lifetime,
                max_lifetime: lifetime,
                size: rand::gen_range(2.0, 6.0),
            });
        }
    }

    pub fn spawn_laser(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.lasers.push(Laser {
            start,
            end,
            color,
            lifetime: 0.15,
            max_lifetime: 0.15,
        });
    }

    pub fn trigger_shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
    }

    pub fn get_shake_offset(&self) -> Vec2 {
        if self.shake_intensity > 0.0 {
            vec2(
                rand::gen_range(-self.shake_intensity, self.shake_intensity),
                rand::gen_range(-self.shake_intensity, self.shake_intensity)
            )
        } else {
            vec2(0.0, 0.0)
        }
    }
}
