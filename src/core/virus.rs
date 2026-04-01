use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VirusKind {
    Fast, // rapide, petit mot
    Classic, // vitesse moyenne, mot moyen
    Heavy, // lent, mot compliqué
    Boss,
    SummonerBoss,
    ReverseBoss,
}

impl VirusKind {
    pub fn base_stats(self) -> (f32, u32) {
        match self {
            Self::Fast => (120.0, 1),
            Self::Classic => (70.0, 2),
            Self::Heavy => (40.0, 4),
            Self::Boss => (25.0, 10),
            Self::SummonerBoss => (20.0, 1),
            Self::ReverseBoss => (22.0, 1),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Fast => 28.0,
            Self::Classic => 36.0,
            Self::Heavy => 54.0,
            Self::Boss => 70.0,
            Self::SummonerBoss => 70.0,
            Self::ReverseBoss => 70.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Fast => GREEN,
            Self::Classic => RED,
            Self::Heavy => ORANGE,
            Self::Boss => PURPLE,
            Self::SummonerBoss => BLUE,
            Self::ReverseBoss => SKYBLUE,
        }
    }
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
        let (speed, health) = kind.base_stats();

        Self { position, kind, speed, health, word }
    }

    pub fn update(&mut self, dt: f32, target: Vec2) {
        let direction = (target - self.position).normalize_or_zero();
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
            VirusKind::SummonerBoss => &assets.virus_boss,
            VirusKind::ReverseBoss => &assets.virus_boss,
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
        self.kind.radius()
    }

    pub fn color(&self) -> Color {
        self.kind.color()
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    pub fn kill(&mut self) {
        self.health = 0;
    }

    // Distance au centre — utile pour savoir si le virus a atteint le joueur
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