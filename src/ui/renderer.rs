use macroquad::prelude::*;

use crate::core::player::Player;

// Toutes les fonctions ici sont du rendu pur — aucune logique métier,
// aucune mutation d'état. Uniquement des draw_*.

pub fn draw_hud(player: &Player, wave_number: u32) {
    draw_health_bar(player);
    draw_wave_number(wave_number);
}

fn draw_health_bar(player: &Player) {
    let bar_width = 200.0;
    let bar_height = 16.0;
    let x = 20.0;
    let y = 20.0;

    let health_ratio = player.health.0 as f32 / 100.0;

    // Fond de la barre (rouge sombre)
    draw_rectangle(x, y, bar_width, bar_height, DARKGRAY);

    // Barre de vie (verte si > 50%, orange sinon)
    let color = if health_ratio > 0.5 { GREEN } else { ORANGE };
    draw_rectangle(x, y, bar_width * health_ratio, bar_height, color);

    // Contour
    draw_rectangle_lines(x, y, bar_width, bar_height, 2.0, WHITE);

    // Texte
    draw_text(
        &format!("HP: {}", player.health.0),
        x + bar_width + 10.0,
        y + bar_height - 2.0,
        16.0,
        WHITE,
    );
}

fn draw_wave_number(wave_number: u32) {
    let text = format!("Vague {}", wave_number);
    let x = screen_width() - 120.0;
    let y = 36.0;
    draw_text(&text, x, y, 20.0, WHITE);
}

// Affiche le mot d'un virus avec les lettres déjà tapées en vert
pub fn draw_virus_word(typed: &str, remaining: &str, x: f32, y: f32) {
    let font_size = 18.0;

    // Partie déjà tapée en vert
    draw_text(typed, x, y, font_size, GREEN);

    // Mesure la largeur réelle du texte tapé pour positionner le reste
    let typed_width = measure_text(typed, None, font_size as u16, 1.0).width;
    draw_text(remaining, x + typed_width, y, font_size, WHITE);
}