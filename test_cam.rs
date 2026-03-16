use macroquad::prelude::*;
#[macroquad::main("TestCam")]
async fn main() {
    loop {
        clear_background(BLACK);
        
        let mut cam = Camera2D {
            zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
            target: vec2(screen_width() / 2.0, screen_height() / 2.0),
            ..Default::default()
        };
        set_camera(&cam);
        
        draw_text("HELLO WORLD", 20.0, 20.0, 30.0, WHITE);
        
        set_default_camera();
        next_frame().await;
    }
}
