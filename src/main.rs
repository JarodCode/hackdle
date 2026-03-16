mod accounts;
mod core;
mod data;
mod ui;

use core::game::Game;
use macroquad::prelude::*;

#[macroquad::main("Hackdle")]
async fn main() {
    let mut game = Game::new().await;

    loop {
        let dt = get_frame_time();

        game.update(dt);
        game.draw();

        next_frame().await;
    }
}