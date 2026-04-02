use macroquad::prelude::*;

pub struct Health(pub u32); // on crée un struct Health pour ne pas le confondre avec d'autre var -> .0 pour accéder à Health (newTypes)

pub struct Player {
    pub position: Vec2,
    pub health: Health,
}

impl Player {
    // Initialise un joueur avec les stats de départ
    pub fn new() -> Self {
        Self {
            // On positionne le joueur au centre de l'écran
            position: Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
            health: Health(100), // 100 pv de base, permet de changer la difficulté (future implémentation)
        }
    }

    // Point d'extension pour une future logique de déplacement/abilities
    pub fn update(&mut self, _dt: f32) {
        // Placeholder si le joueur bouge dans le futur
    }

    // Rendu du joueur avec prise en compte de l'offset global (shake/caméra) (devrait être dans renderer)
    pub fn draw(&self, assets: &crate::ui::assets::GameAssets, offset: Vec2) {
        // La largeur souhaitée
        let target_width = 280.0;
        
        // On récupère les dimensions d'origine de l'image
        let tex_w = assets.player.width();
        let tex_h = assets.player.height();
        
        // On calcule la hauteur pour garder les proportions exactes
        let target_height = target_width * (tex_h / tex_w);

        draw_texture_ex(
            &assets.player,
            // On centre l'image en utilisant les nouvelles dimensions calculées
            self.position.x - target_width / 2.0 + offset.x,
            self.position.y - target_height / 2.0 + offset.y,
            WHITE,
            DrawTextureParams {
                // On applique les dimensions proportionnelles ici
                dest_size: Some(vec2(target_width, target_height)),
                ..Default::default()
            },
        );
    }

    // Utilisé par les transitions d'état (InWave -> GameOver)
    pub fn is_alive(&self) -> bool {
        self.health.0 > 0
    }

    // Applique des dégâts sans jamais passer sous 0
    pub fn take_damage(&mut self, amount: u32) {
        self.health.0 = self.health.0.saturating_sub(amount); // saturing_sub = bloque la soustraction à 0
    }
}