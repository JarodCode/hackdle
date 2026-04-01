use macroquad::prelude::*;

#[derive(PartialEq)]

// Différents types de virus pour varier le gameplay
pub enum VirusKind {
    Fast, // rapide, petit mot
    Classic, // vitesse moyenne, mot moyen
    Heavy, // lent, mot compliqué
    Boss,
}

pub struct Virus {
    pub position: Vec2,
    pub kind: VirusKind,
    pub speed: f32, // pixels par seconde
    pub health: u32,
    pub word: String, // mot à taper pour éliminer
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
        let direction = (target - self.position).normalize(); // vecteur pointant du virus vers le joueur (longueur 1 car normalisé)
        self.position += direction * self.speed * dt; // on déplace le virus à vitesse .speed (dt : rend le déplacement indépendant du framerate)
    }

    pub fn draw(&self, assets: &crate::core::assets::GameAssets) {
        self.draw_with_offset(assets, 0.0, 0.0, WHITE);
    }

    pub fn draw_with_offset(&self, assets: &crate::core::assets::GameAssets, offset_x: f32, offset_y: f32, color_override: Color) {
        let tex = match self.kind {
            VirusKind::Fast => &assets.virus_fast,
            VirusKind::Classic => &assets.virus_classic,
            VirusKind::Heavy => &assets.virus_heavy,
            VirusKind::Boss => &assets.virus_boss,
        };

        let radius = self.radius();
        let size = radius * 2.0;

        draw_texture_ex(
            tex,
            self.position.x - radius + offset_x,
            self.position.y - radius + offset_y,
            color_override,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }

    pub fn radius(&self) -> f32 {
        match self.kind {
            VirusKind::Fast    => 28.0,
            VirusKind::Classic => 36.0,
            VirusKind::Heavy   => 55.0,
            VirusKind::Boss    => 64.0,
        }
    }

    pub fn color(&self) -> Color {
        match self.kind {
            VirusKind::Fast    => GREEN,
            VirusKind::Classic => RED,
            VirusKind::Heavy   => ORANGE,
            VirusKind::Boss    => PURPLE,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    pub fn distance_to(&self, target: Vec2) -> f32 {
        (target - self.position).length()
    }

    // Fait rebondir le virus quand il touche le joueur
    pub fn bounce_away(&mut self, target: Vec2) {
        let direction = (self.position - target)
            .try_normalize()
            .unwrap_or(Vec2::new(1.0, 0.0)); // None = (1,0) (droite), cas où le virus est exactement sur le joueur
        let bounce_distance = self.speed * 1.5;
        self.position += direction * bounce_distance;
    }
}