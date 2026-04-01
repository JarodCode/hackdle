use macroquad::prelude::*;

use crate::accounts::UserProfile;
use crate::ui::assets::GameAssets;
use crate::core::boss;
use crate::core::player::Player;
use crate::core::virus::VirusKind;
use crate::core::wave::VirusEntry;

pub fn draw_hud(player: &Player, wave_number: u32, font: Option<&Font>) {
    draw_health_bar(player, font);
    draw_wave_number(wave_number, font);
}

fn draw_health_bar(player: &Player, font: Option<&Font>) {
    let x = 20.0;
    let y = 20.0;

    let health_ratio = player.health.0 as f32 / 100.0;

    let color = if health_ratio > 0.75 {
        GREEN
    } else if health_ratio > 0.5 {
        YELLOW
    } else if health_ratio > 0.25 {
        ORANGE
    } else {
        RED
    };

    let text = format!("SYS.INTEGRITY {:03}%", player.health.0);
    draw_text_ex(
        &text,
        x,
        y + 16.0,
        TextParams { font_size: 20, font, color, ..Default::default() },
    );

    let bar_width = 200.0;
    let bar_height = 10.0;
    let bar_y = y + 24.0;

    draw_rectangle(x, bar_y, bar_width, bar_height, DARKGRAY);
    draw_rectangle(x, bar_y, bar_width * health_ratio, bar_height, color);
    draw_rectangle_lines(x, bar_y, bar_width, bar_height, 2.0, WHITE);
}

fn draw_wave_number(wave_number: u32, font: Option<&Font>) {
    let text = format!("WAVE_ID: {:03}", wave_number);
    let x = screen_width() - 160.0;
    let y = 36.0;
    draw_text_ex(
        &text,
        x,
        y,
        TextParams { font_size: 20, font, color: WHITE, ..Default::default() },
    );
}

// Affiche le mot d'un virus avec les lettres déjà tapées en vert
pub fn draw_virus_word(typed: &str, remaining: &str, x: f32, y: f32, font: Option<&Font>) {
    let font_size = 18.0;

    draw_text_ex(typed, x, y, TextParams { font_size: font_size as u16, font, color: GREEN, ..Default::default() });

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

// --- Rendu de la vague (extrait de wave.rs) ---

pub fn draw_wave(
    entries: &[VirusEntry],
    killed: usize,
    to_kill: usize,
    assets: &GameAssets,
    global_offset: Vec2,
) {
    for entry in entries.iter() {
        let mut offset_x = global_offset.x;
        let mut offset_y = global_offset.y;
        let mut color_override = WHITE;

        if entry.glitch_timer > 0.0 {
            offset_x += rand::gen_range(-5.0, 5.0);
            offset_y += rand::gen_range(-5.0, 5.0);
            color_override = if rand::gen_range(0, 2) == 0 { RED } else { BLUE };
        }

        entry.virus.draw_with_offset(assets, offset_x, offset_y, color_override);

        let x = entry.virus.position.x - 20.0 + offset_x;
        let y = entry.virus.position.y - entry.virus.radius() - 8.0 + offset_y;

        if matches!(entry.virus.kind, VirusKind::ReverseBoss) {
            let visible = boss::visual_word(entry.virus.kind, &entry.virus.word);
            draw_virus_word("", &visible, x, y, Some(&assets.font));
        } else if entry.active {
            draw_virus_word(
                entry.typing.typed_part(),
                entry.typing.remaining_part(),
                x, y,
                Some(&assets.font),
            );
        } else {
            draw_virus_word("", &entry.virus.word, x, y, Some(&assets.font));
        }
    }

    draw_kill_counter(killed, to_kill);
}

fn draw_kill_counter(killed: usize, to_kill: usize) {
    let text = format!("{} / {}", killed, to_kill);
    let x = screen_width() / 2.0 - 30.0;
    let y = screen_height() - 16.0;
    draw_text(&text, x, y, 20.0, YELLOW);
}