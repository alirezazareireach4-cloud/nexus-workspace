use macroquad::prelude::*;

use crate::components::{
    background::draw_background,
    sidebar::draw_sidebar,
    topbar::draw_topbar,
    card::draw_cards,
};
use crate::ui::file_manager;
use crate::ui::theme::Theme;


pub async fn show() {
    let theme = Theme::load().await;

    loop {
        clear_background(Color::from_rgba(7, 10, 20, 255));

        // Background
        draw_background();

        // Components
        draw_topbar(&theme);

        draw_sidebar(&theme);

        draw_cards(&theme);

        next_frame().await;
        
        let clicked = draw_sidebar(&theme);
        if clicked{
            file_manager::show().await;
        }
    }
}