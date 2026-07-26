use macroquad::prelude::*;
use chrono::{Local, DateTime};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Clone)]
pub struct NotificationItem {
    pub message: String,
    pub timestamp: String,
    pub kind: NotificationKind,
}

#[derive(Clone, PartialEq)]
pub enum NotificationKind {
    Info,
    Success,
    Warning,
    Music,
}

pub struct NotificationCenter {
    pub notifications: Vec<NotificationItem>,
    max_history: usize,
    log_file_path: String,
}

impl NotificationCenter {
    pub fn new() -> Self {
        let log_file_path = "workspace_activity.log".to_string();


        if !Path::new(&log_file_path).exists() {
            let _ = File::create(&log_file_path);
        }

        let mut center = Self {
            notifications: Vec::new(),
            max_history: 100,
            log_file_path,
        };

        center.push("System Workspace initialized successfully.", NotificationKind::Success);
        center
    }


    pub fn push(&mut self, msg: &str, kind: NotificationKind) {
        let now: DateTime<Local> = Local::now();
        let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let kind_label = match kind {
            NotificationKind::Success => "[SUCCESS]",
            NotificationKind::Warning => "[WARNING]",
            NotificationKind::Music   => "[MUSIC]",
            NotificationKind::Info    => "[INFO]",
        };


        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            let log_line = format!("[{}] {} {}\n", time_str, kind_label, msg);
            let _ = file.write_all(log_line.as_bytes());
        }


        let short_time_str = now.format("%H:%M:%S").to_string();
        self.notifications.insert(0, NotificationItem {
            message: msg.to_string(),
            timestamp: short_time_str,
            kind,
        });

        if self.notifications.len() > self.max_history {
            self.notifications.pop();
        }
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
        self.push("Notification history cleared by user.", NotificationKind::Info);
    }

    pub fn update(&mut self, content_x: f32) {
        draw_text("🔔 Notification Center & Activity History", content_x, 95.0, 20.0, MAGENTA);
        draw_text("Tracks all background workspace activities, file operations, and audio events in real-time.", content_x, 120.0, 13.0, SKYBLUE);


        let clear_btn = Rect::new(content_x + 480.0, 100.0, 140.0, 30.0);
        let mouse_pos = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);
        let btn_hovered = clear_btn.contains(mouse_pos.into());

        let btn_col = if btn_hovered { Color::new(0.5, 0.15, 0.2, 1.0) } else { Color::new(0.35, 0.1, 0.15, 1.0) };
        draw_rectangle(clear_btn.x, clear_btn.y, clear_btn.w, clear_btn.h, btn_col);
        draw_rectangle_lines(clear_btn.x, clear_btn.y, clear_btn.w, clear_btn.h, 1.0, Color::new(0.6, 0.2, 0.3, 1.0));
        draw_text("Clear History", clear_btn.x + 28.0, clear_btn.y + 20.0, 13.0, WHITE);

        if btn_hovered && clicked {
            self.clear();
        }

        let ty = 155.0;
        let panel_h = screen_height() - 200.0;
        let list_rect = Rect::new(content_x, ty, 620.0, panel_h);

        draw_rectangle(list_rect.x, list_rect.y, list_rect.w, list_rect.h, Color::new(0.1, 0.07, 0.18, 1.0));
        draw_rectangle_lines(list_rect.x, list_rect.y, list_rect.w, list_rect.h, 1.0, Color::new(0.3, 0.2, 0.4, 1.0));

        if self.notifications.is_empty() {
            draw_text("No activity recorded yet.", content_x + 20.0, ty + 35.0, 14.0, GRAY);
        } else {
            let mut item_y = ty + 15.0;
            let max_visible = (panel_h - 20.0) / 45.0;

            for (idx, item) in self.notifications.iter().enumerate() {
                if idx as f32 >= max_visible {
                    break;
                }

                let item_rect = Rect::new(content_x + 10.0, item_y, 600.0, 38.0);
                let item_hovered = item_rect.contains(mouse_pos.into());

                let bg_color = if item_hovered {
                    Color::new(0.18, 0.12, 0.28, 1.0)
                } else {
                    Color::new(0.13, 0.09, 0.22, 1.0)
                };

                let accent_color = match item.kind {
                    NotificationKind::Success => GREEN,
                    NotificationKind::Warning => YELLOW,
                    NotificationKind::Music => MAGENTA,
                    NotificationKind::Info => SKYBLUE,
                };

                draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg_color);

                draw_rectangle(item_rect.x, item_rect.y, 4.0, item_rect.h, accent_color);
                draw_rectangle_lines(item_rect.x, item_rect.y, item_rect.w, item_rect.h, 1.0, Color::new(0.25, 0.18, 0.35, 1.0));


                let time_text = format!("[{}]", item.timestamp);
                draw_text(&time_text, item_rect.x + 15.0, item_rect.y + 24.0, 12.0, LIGHTGRAY);
                draw_text(&item.message, item_rect.x + 85.0, item_rect.y + 24.0, 13.0, WHITE);

                item_y += 45.0;
            }
        }

        draw_text("Saved to 'workspace_activity.log' | Press [ESC] to return", content_x, screen_height() - 15.0, 12.0, GRAY);
    }
}