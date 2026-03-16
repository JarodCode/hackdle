use macroquad::prelude::*;

#[derive(PartialEq)]
pub enum VirusKind {
    Fast,
    Classic,
    Heavy,
    Boss,
}

pub struct Virus {
    pub position: Vec2,
    pub kind: VirusKind,
    pub speed: f32,      // pixels par seconde
    pub health: u32,
    pub word: String,    // mot à taper pour l'éliminer
}

impl Virus {
    pub fn new(position: Vec2, kind: VirusKind, word: String) -> Self {
        // Les stats varient selon le type d'ennemi
        let (speed, health) = match kind {
            VirusKind::Fast    => (120.0, 1),
            VirusKind::Classic => (70.0,  2),
            VirusKind::Heavy   => (40.0,  4),
            VirusKind::Boss    => (25.0, 10),
        };

        Self { position, kind, speed, health, word }
    }

    pub fn update(&mut self, dt: f32, target: Vec2) {
        // Calcule la direction vers le joueur et avance
        let direction = (target - self.position).normalize();
        self.position += direction * self.speed * dt;
    }

    pub fn draw(&self) {
        let color = match self.kind {
            VirusKind::Fast    => GREEN,
            VirusKind::Classic => RED,
            VirusKind::Heavy   => ORANGE,
            VirusKind::Boss    => PURPLE,
        };

        // Placeholder visuel : un cercle coloré
        // Le mot est affiché par wave.rs qui connaît l'état de frappe
        draw_circle(self.position.x, self.position.y, self.radius(), color);
    }

    pub fn radius(&self) -> f32 {
        match self.kind {
            VirusKind::Fast    => 12.0,
            VirusKind::Classic => 18.0,
            VirusKind::Heavy   => 26.0,
            VirusKind::Boss    => 40.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    // Distance au centre — utile pour savoir si le virus a atteint le joueur
    pub fn distance_to(&self, target: Vec2) -> f32 {
        (target - self.position).length()
    }
}