use macroquad::prelude::*;

use crate::accounts::UserProfile;
use crate::core::player::Player;

// Toutes les fonctions ici sont du rendu pur — aucune logique métier,
// aucune mutation d'état. Uniquement des draw_*.

pub fn draw_hud(player: &Player, wave_number: u32, font: Option<&Font>) {
    draw_health_bar(player, font);
    draw_wave_number(wave_number, font);
}

fn draw_health_bar(player: &Player, font: Option<&Font>) {
    let x = 20.0;
    let y = 20.0;

    let health_ratio = player.health.0 as f32 / 100.0;

    // Stylized Cyberpunk Health Bar
    let bars_total = 10;
    let bars_active = (health_ratio * bars_total as f32).ceil() as i32;
    
    let mut bar_content = String::new();
    for i in 0..bars_total {
        if i < bars_active {
            bar_content.push('|');
        } else {
            bar_content.push(' ');
        }
    }
    
    let color = if health_ratio > 0.5 { GREEN } else if health_ratio > 0.25 { YELLOW } else { RED };
    let text = format!("SYS.INTEGRITY [{}] {:03}%", bar_content, player.health.0);
    
    draw_text_ex(
        &text,
        x,
        y + 16.0,
        TextParams {
            font_size: 20,
            font,
            color,
            ..Default::default()
        },
    );
}

fn draw_wave_number(wave_number: u32, font: Option<&Font>) {
    let text = format!("WAVE_ID: {:03}", wave_number);
    let x = screen_width() - 160.0;
    let y = 36.0;
    draw_text_ex(
        &text,
        x,
        y,
        TextParams {
            font_size: 20,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
}

// Affiche le mot d'un virus avec les lettres déjà tapées en vert
pub fn draw_virus_word(typed: &str, remaining: &str, x: f32, y: f32, font: Option<&Font>) {
    let font_size = 18.0;

    // Partie déjà tapée en vert
    draw_text_ex(typed, x, y, TextParams { font_size: font_size as u16, font, color: GREEN, ..Default::default() });

    // Mesure la largeur réelle du texte tapé pour positionner le reste
    let typed_width = measure_text(typed, font, font_size as u16, 1.0).width;
    draw_text_ex(remaining, x + typed_width, y, TextParams { font_size: font_size as u16, font, color: WHITE, ..Default::default() });
}

pub fn draw_scoreboard(entries: &[UserProfile], title: &str, max_rows: usize, font: Option<&Font>) {
    let panel_width = 280.0;
    let padding = 16.0;
    let x = screen_width() - panel_width - padding;
    let y = 80.0;

    draw_rectangle_lines(x - 10.0, y - 40.0, panel_width + 20.0, 220.0, 2.0, DARKGRAY);
    draw_text_ex(title, x, y - 10.0, TextParams { font_size: 24, font, color: YELLOW, ..Default::default() });

    let line_height = 24.0;
    for (idx, profile) in entries.iter().take(max_rows).enumerate() {
        let avg = profile.average_wave();
        let line = format!(
            "{}. {:<10} — W{:02} (avg {:.1})",
            idx + 1,
            profile.username,
            profile.best_wave,
            avg
        );
        draw_text_ex(&line, x, y + line_height * (idx as f32 + 1.0), TextParams { font_size: 18, font, color: WHITE, ..Default::default() });
    }

    if entries.is_empty() {
        draw_text_ex("NO_AGENTS_FOUND", x, y + line_height, TextParams { font_size: 18, font, color: GRAY, ..Default::default() });
    }
}