use macroquad::prelude::*;

pub fn draw_background() {
    // Background
    draw_rectangle(25.0,0.0,screen_width(),screen_height(),WHITE,);


    // Grid Dots
    for x in (0..screen_width() as i32).step_by(40) {
        for y in (0..screen_height() as i32).step_by(40) {
            draw_circle(
                x as f32,
                y as f32,
                1.2,
                Color::from_rgba(255, 255, 255, 18),
            );
        }
    }

    // Glow Top Right
    draw_circle(
        screen_width() - 150.0,
        120.0,
        260.0,
        Color::from_rgba(70, 90, 255, 18),
    );

    // Glow Bottom Left
    draw_circle(
        180.0,
        screen_height() - 120.0,
        220.0,
        Color::from_rgba(170, 0, 255, 15),
    );
}