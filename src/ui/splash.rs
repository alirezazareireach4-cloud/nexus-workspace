use macroquad::prelude::*;
pub async fn show(){
    let logo = load_texture("assets/images/logo.png").await.unwrap();
    let start_time = get_time();
    loop{
        clear_background(WHITE);
        let logo_x = screen_width() / 2.0 - logo.width() / 2.0;
        let logo_y = screen_height() / 2.0 - logo.height() / 2.0;


        draw_texture(&logo,logo_x,logo_y,WHITE);
        for x in (0..screen_width() as i32).step_by(40) {
            for y in (0..screen_height() as i32).step_by(40) {
                draw_circle(x as f32,y as f32,1.5,Color::from_rgba(255, 255, 255, 120),);}}
        next_frame().await;

        if get_time() - start_time >= 3.0{
            break;
        }
    }
}