use macroquad::prelude::*;

use crate::ui::theme::Theme;

pub fn draw_cards(theme: &Theme) {
    let cards = [
        ("File Manager", "12 Active", Color::from_rgba(59, 130, 246, 255)),
        ("Music Library", "34 Pending", Color::from_rgba(239, 68, 68, 255)),
        ("File Transfer", "78%", Color::from_rgba(34, 197, 94, 255)),
        ("Notes", "8 Online", Color::from_rgba(245, 158, 11, 255)),
        ("AI Assistant", "Protected", Color::from_rgba(139, 92, 246, 255)),
        ("Settings", "Running", Color::from_rgba(6, 182, 212, 255)),
    ];

    let (mx, my) = mouse_position();

    let start_x = 300.0;
    let start_y = 120.0;

    let width = 240.0;
    let height = 145.0;

    for (i, card) in cards.iter().enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;

        let mut x = start_x + col * 270.0;
        let mut y = start_y + row * 190.0;

        let mut w = width;
        let mut h = height;

        let hovered =
            mx >= x &&
                mx <= x + width &&
                my >= y &&
                my <= y + height;

        if hovered {
            x -= 5.0;
            y -= 5.0;
            w += 10.0;
            h += 10.0;
        }

        // Shadow
        draw_rectangle(
            x + 8.0,
            y + 8.0,
            w,
            h,
            Color::from_rgba(0, 0, 0, 35),
        );

        // Card
        draw_rectangle(
            x,
            y,
            w,
            h,
            WHITE,
        );

        // Color Bar
        draw_rectangle(
            x,
            y,
            10.0,
            h,
            card.2,
        );

        // Title
        draw_text_ex(
            card.0,
            x + 28.0,
            y + 45.0,
            TextParams {
                font: Some(&theme.inter_semibold),
                font_size: 26,
                color: BLACK,
                ..Default::default()
            },
        );

        // Subtitle
        draw_text_ex(
            card.1,
            x + 28.0,
            y + 82.0,
            TextParams {
                font: Some(&theme.inter_regular),
                font_size: 20,
                color: DARKGRAY,
                ..Default::default()
            },
        );

        // Status circle
        draw_circle(
            x + w - 30.0,
            y + 30.0,
            8.0,
            card.2,
        );
    }
}