use macroquad::prelude::*;

// Newtype pour éviter de confondre la vie avec un u32 quelconque
pub struct Health(pub u32);

pub struct Player {
    pub position: Vec2,
    pub health: Health,
}

impl Player {
    pub fn new() -> Self {
        Self {
            // Centré à l'écran — screen_width/height sont disponibles dès que
            // macroquad est initialisé (donc après Game::new())
            position: Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
            health: Health(100),
        }
    }

    pub fn update(&mut self, _dt: f32) {
        // Placeholder — le joueur ne bouge pas, c'est les virus qui se déplacent vers lui
    }

    pub fn draw(&self, assets: &crate::core::assets::GameAssets, offset: Vec2) {
        let size = 40.0;

        draw_texture_ex(
            &assets.player,
            self.position.x - size / 2.0 + offset.x,
            self.position.y - size / 2.0 + offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }

    pub fn is_alive(&self) -> bool {
        self.health.0 > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health.0 = self.health.0.saturating_sub(amount);
    }
}