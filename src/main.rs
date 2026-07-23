mod ui;
mod components;

use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf{
        window_title : "Nexus Workspace".to_owned(),
        fullscreen:true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::run().await;
}