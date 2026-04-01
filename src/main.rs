mod accounts;
mod core;
mod data;
mod ui;

use core::game::Game;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Hackdle".to_string(),
        fullscreen: true,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        let dt = get_frame_time();

        game.update(dt);
        game.draw();

        next_frame().await;
    }
}