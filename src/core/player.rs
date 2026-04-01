use macroquad::prelude::*;

pub struct Health(pub u32); // on crée un struct Health pour ne pas le confondre avec d'autre var -> .0 pour accéder à Health

pub struct Player {
    pub position: Vec2,
    pub health: Health,
}

impl Player {
    pub fn new() -> Self {
        Self {
            // On positionne le joueur au centre de l'écran (pb si on change la taille de l'écran)
            position: Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
            health: Health(100), // 100 pv de base.
        }
    }

    pub fn update(&mut self, _dt: f32) {
        // Placeholder si le joueur bouge dans le futur
    }

    pub fn draw(&self) {
        // Placeholder visuel : un cercle blanc au centre
        draw_circle(self.position.x, self.position.y, 20.0, WHITE);
    }

    pub fn is_alive(&self) -> bool {
        self.health.0 > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health.0 = self.health.0.saturating_sub(amount); // saturing_sub = bloque la soustraction à 0
    }
}