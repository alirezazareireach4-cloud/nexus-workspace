use macroquad::prelude::*;

use crate::ui::theme::Theme;



pub async fn show() {

    let theme = Theme::load().await;
    let accent = Color::from_rgba(99,102,241,255);

    let mut search_text = String::new();


    let mut files = vec![
        ("main.rs".to_string(), "Rust File".to_string()),
        ("Cargo.toml".to_string(), "Config".to_string()),
        ("assets".to_string(), "Folder".to_string()),
        ("README.md".to_string(), "Document".to_string()),
        ("theme.rs".to_string(), "Rust File".to_string()),
    ];

    //theme.accent

    loop {

        clear_background(
            Color::from_rgba(245, 246, 250, 255)
        );



        // ================= HEADER =================

        draw_text(
            "File Manager",
            50.0,
            70.0,
            38.0,
            BLACK
        );


        draw_text(
            "Manage your workspace files",
            50.0,
            100.0,
            18.0,
            GRAY
        );



        // ================= SEARCH BAR =================


        draw_rectangle(
            820.0,
            35.0,
            380.0,
            45.0,
            WHITE
        );


        draw_rectangle_lines(
            820.0,
            35.0,
            380.0,
            45.0,
            2.0,
            accent
        );



        draw_text(
            if search_text.is_empty() {
                "Search..."
            }
            else {
                &search_text
            },
            840.0,
            64.0,
            20.0,
            GRAY
        );



        // گرفتن ورودی سرچ

        if is_key_pressed(KeyCode::Backspace) {

            search_text.pop();

        }


        while let Some(c) = get_char_pressed() {

            if search_text.len() < 25 {

                search_text.push(c);

            }

        }



        // ================= FILE PANEL =================


        draw_rectangle(
            780.0,
            130.0,
            560.0,
            500.0,
            WHITE
        );


        draw_text(
            "Files",
            820.0,
            175.0,
            25.0,
            BLACK
        );



        let mut x = 820.0;
        let mut y = 210.0;



        for (index, file) in files.iter().enumerate() {


            if !search_text.is_empty()
                &&
                !file.0
                    .to_lowercase()
                    .contains(
                        &search_text.to_lowercase()
                    )
            {
                continue;
            }



            let width = 150.0;
            let height = 120.0;



            let mouse = mouse_position();



            let hover =
                mouse.0 >= x &&
                    mouse.0 <= x + width &&
                    mouse.1 >= y &&
                    mouse.1 <= y + height;



            let color =
                if hover {
                    Color::from_rgba(
                        230,
                        235,
                        255,
                        255
                    )
                }
                else {
                    Color::from_rgba(
                        250,
                        250,
                        252,
                        255
                    )
                };



            draw_rectangle(
                x,
                y,
                width,
                height,
                color
            );



            draw_rectangle(
                x + 20.0,
                y + 15.0,
                40.0,
                40.0,
                accent
            );



            draw_text(
                &file.0,
                x + 15.0,
                y + 80.0,
                18.0,
                BLACK
            );


            draw_text(
                &file.1,
                x + 15.0,
                y + 105.0,
                14.0,
                GRAY
            );



            if hover {

                draw_rectangle_lines(
                    x,
                    y,
                    width,
                    height,
                    2.0,
                    accent
                );

            }



            x += 170.0;


            if x > 1200.0 {

                x = 820.0;
                y += 150.0;

            }



            // کلیک روی فایل

            if hover && is_mouse_button_pressed(MouseButton::Left) {

                println!("Selected file: {}", file.0);

            }


        }
        // ================= ACTION BUTTONS =================


        draw_rectangle(
            820.0,
            660.0,
            120.0,
            45.0,
            accent
        );


        draw_text(
            "Rename",
            845.0,
            690.0,
            20.0,
            WHITE
        );



        draw_rectangle(
            960.0,
            660.0,
            120.0,
            45.0,
            Color::from_rgba(
                220,
                70,
                70,
                255
            )
        );


        draw_text(
            "Delete",
            995.0,
            690.0,
            20.0,
            WHITE
        );



        // ================= BUTTON HOVER =================


        let mouse = mouse_position();



        let rename_hover =
            mouse.0 >= 820.0 &&
                mouse.0 <= 940.0 &&
                mouse.1 >= 660.0 &&
                mouse.1 <= 705.0;



        let delete_hover =
            mouse.0 >= 960.0 &&
                mouse.0 <= 1080.0 &&
                mouse.1 >= 660.0 &&
                mouse.1 <= 705.0;



        if rename_hover {

            draw_rectangle_lines(
                820.0,
                660.0,
                120.0,
                45.0,
                2.0,
                WHITE
            );

        }



        if delete_hover {

            draw_rectangle_lines(
                960.0,
                660.0,
                120.0,
                45.0,
                2.0,
                WHITE
            );

        }



        // ================= BUTTON ACTIONS =================



        if rename_hover
            &&
            is_mouse_button_pressed(
                MouseButton::Left
            )
        {

            println!("Rename clicked");

        }



        if delete_hover
            &&
            is_mouse_button_pressed(
                MouseButton::Left
            )
        {

            println!("Delete clicked");

        }



        // ================= FOOTER =================


        draw_text(
            "Nexus Workspace File Manager",
            50.0,
            720.0,
            16.0,
            GRAY
        );



        next_frame().await;

    }

}