use macroquad::prelude::*;

use crate::ui::theme::Theme;

pub fn draw_topbar(theme: &Theme) {
    // Topbar Background
    draw_rectangle(
        250.0,
        0.0,
        screen_width() - 250.0,
        80.0,
        WHITE,
    );

    // Bottom Border
    draw_rectangle(
        250.0,
        79.0,
        screen_width() - 250.0,
        1.5,
        Color::from_rgba(220, 220, 220, 255),
    );

    // Page Title
    draw_text_ex(
        "Dashboard",
        290.0,
        48.0,
        TextParams {
            font: Some(&theme.inter_bold),
            font_size: 32,
            color: BLACK,
            ..Default::default()
        },
    );

    // Search Box
    let search_x = screen_width() / 2.0 - 170.0;

    draw_rectangle(
        search_x,
        20.0,
        340.0,
        40.0,
        Color::from_rgba(245, 245, 245, 255),
    );



    // Notification Circle
    draw_circle(
        screen_width() - 180.0,
        40.0,
        8.0,
        RED,
    );

    // Online Status
    draw_circle(
        screen_width() - 120.0,
        40.0,
        7.0,
        GREEN,
    );

    draw_text_ex(
        "Online",
        screen_width() - 105.0,
        46.0,
        TextParams {
            font: Some(&theme.inter_medium),
            font_size: 20,
            color: DARKGRAY,
            ..Default::default()
        },
    );

    // User
    draw_text_ex(
        "Alireza",
        screen_width() - 260.0,
        46.0,
        TextParams {
            font: Some(&theme.inter_medium),
            font_size: 20,
            color: BLACK,
            ..Default::default()
        },
    );
}