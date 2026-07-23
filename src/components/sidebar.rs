use macroquad::prelude::*;

use crate::ui::theme::Theme;


pub fn draw_sidebar(theme: &Theme) -> bool {

    // Background
    draw_rectangle(
        0.0,
        0.0,
        250.0,
        screen_height(),
        Color::from_rgba(15, 15, 18, 255),
    );


    // Logo
    draw_text_ex(
        "NEXUS",
        35.0,
        55.0,
        TextParams {
            font: Some(&theme.inter_bold),
            font_size: 34,
            color: WHITE,
            ..Default::default()
        },
    );


    draw_text_ex(
        "Workspace",
        35.0,
        82.0,
        TextParams {
            font: Some(&theme.inter_regular),
            font_size: 18,
            color: GRAY,
            ..Default::default()
        },
    );



    let menu = [
        "Dashboard",
        "File Manager",
        "Music Library",
        "File Transfer",
        "Notes",
        "AI Assistant",
        "Settings",
    ];



    let (mx, my) = mouse_position();


    let mut y = 160.0;


    let mut file_manager_clicked = false;



    for item in menu {


        let hovered =
            mx >= 20.0 &&
                mx <= 220.0 &&
                my >= y - 30.0 &&
                my <= y + 10.0;



        let color =
            if hovered {
                RED
            } else {
                WHITE
            };



        draw_text_ex(
            item,
            40.0,
            y,
            TextParams {

                font:
                Some(&theme.inter_medium),

                font_size:
                24,

                color,

                ..Default::default()
            },
        );



        if hovered
            && is_mouse_button_pressed(MouseButton::Left)
            && item == "File Manager"
        {
            file_manager_clicked = true;
        }



        y += 60.0;
    }



    // Version

    draw_text_ex(
        "v0.1 Alireza",
        35.0,
        screen_height() - 30.0,
        TextParams {

            font:
            Some(&theme.inter_regular),

            font_size:
            18,

            color:
            DARKGRAY,

            ..Default::default()
        },
    );



    file_manager_clicked
}