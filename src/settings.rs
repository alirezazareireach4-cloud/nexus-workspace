use macroquad::prelude::*;
use std::fs;
use std::path::Path;
use crate::notifications::{NotificationCenter, NotificationKind};

#[derive(Clone)]
pub struct SettingsConfig {
    pub theme_dark: bool,
    pub master_volume: f32,
    pub download_path: String,
    pub auto_scan_music: bool,
    pub network_port: u16,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            theme_dark: true,
            master_volume: 80.0,
            download_path: "downloads".to_string(),
            auto_scan_music: true,
            network_port: 8889,
        }
    }
}

pub struct SettingsState {
    pub config: SettingsConfig,
    pub status_message: String,
    config_file_path: String,
}

impl SettingsState {
    pub fn new() -> Self {
        let config_file_path = "settings.json".to_string();
        let mut state = Self {
            config: SettingsConfig::default(),
            status_message: "Settings loaded with default values.".to_string(),
            config_file_path,
        };
        state.load_from_disk();
        state
    }

    pub fn load_from_disk(&mut self) {
        if Path::new(&self.config_file_path).exists() {
            if let Ok(data) = fs::read_to_string(&self.config_file_path) {
                if let Ok(parsed) = serde_json_from_str_custom(&data) {
                    self.config = parsed;
                    self.status_message = "Settings loaded successfully from disk.".to_string();
                }
            }
        }
    }

    pub fn save_to_disk(&mut self, notifications: &mut NotificationCenter) {
        let serialized = format!(
            "{{\n  \"theme_dark\": {},\n  \"master_volume\": {},\n  \"download_path\": \"{}\",\n  \"auto_scan_music\": {},\n  \"network_port\": {}\n}}",
            self.config.theme_dark,
            self.config.master_volume,
            self.config.download_path,
            self.config.auto_scan_music,
            self.config.network_port
        );

        match fs::write(&self.config_file_path, serialized) {
            Ok(_) => {
                self.status_message = "Settings saved successfully!".to_string();
                notifications.push("Workspace settings updated and saved.", NotificationKind::Success);
            }
            Err(e) => {
                self.status_message = format!("Failed to save settings: {}", e);
                notifications.push("Error saving workspace settings.", NotificationKind::Warning);
            }
        }
    }

    pub fn update(&mut self, content_x: f32, notifications: &mut NotificationCenter) {
        draw_text("⚙️ Workspace Settings & Preferences", content_x, 95.0, 20.0, SKYBLUE);
        draw_text("Customize your workspace environment, paths, and audio preferences:", content_x, 120.0, 13.0, LIGHTGRAY);

        let box_w = 620.0;
        let box_h = 400.0;
        let box_y = 150.0;

        draw_rectangle(content_x, box_y, box_w, box_h, Color::new(0.06, 0.08, 0.14, 1.0));
        draw_rectangle_lines(content_x, box_y, box_w, box_h, 1.5, SKYBLUE);

        let mouse_pos = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        let mut item_y = box_y + 25.0;

        // تنظیم ۱: حالت تاریک / روشن (Theme)
        draw_text("Dark Mode Theme", content_x + 20.0, item_y + 18.0, 14.0, WHITE);
        let theme_btn = Rect::new(content_x + 480.0, item_y, 110.0, 28.0);
        let theme_hover = theme_btn.contains(mouse_pos.into());
        let theme_col = if theme_hover { Color::new(0.2, 0.4, 0.6, 1.0) } else { Color::new(0.15, 0.25, 0.4, 1.0) };

        draw_rectangle(theme_btn.x, theme_btn.y, theme_btn.w, theme_btn.h, theme_col);
        draw_rectangle_lines(theme_btn.x, theme_btn.y, theme_btn.w, theme_btn.h, 1.0, WHITE);
        let theme_text = if self.config.theme_dark { "Enabled [✔]" } else { "Disabled [ ]" };
        draw_text(theme_text, theme_btn.x + 12.0, theme_btn.y + 19.0, 13.0, WHITE);

        if theme_hover && clicked {
            self.config.theme_dark = !self.config.theme_dark;
            self.save_to_disk(notifications);
        }

        item_y += 60.0;

        // تنظیم ۲: اسلایدر ولوم صدا (Master Volume)
        draw_text(&format!("Master Volume: {:.0}%", self.config.master_volume), content_x + 20.0, item_y + 18.0, 14.0, WHITE);
        let slider_x = content_x + 250.0;
        let slider_y = item_y + 8.0;
        let slider_w = 340.0;
        let slider_h = 12.0;

        draw_rectangle(slider_x, slider_y, slider_w, slider_h, Color::new(0.1, 0.15, 0.25, 1.0));
        let fill_w = slider_w * (self.config.master_volume / 100.0);
        draw_rectangle(slider_x, slider_y, fill_w, slider_h, SKYBLUE);
        draw_rectangle_lines(slider_x, slider_y, slider_w, slider_h, 1.0, Color::new(0.3, 0.5, 0.7, 1.0));

        let slider_rect = Rect::new(slider_x, slider_y - 5.0, slider_w, 22.0);
        if slider_rect.contains(mouse_pos.into()) && is_mouse_button_down(MouseButton::Left) {
            let click_x = mouse_pos.0 - slider_x;
            self.config.master_volume = ((click_x / slider_w) * 100.0).clamp(0.0, 100.0);
            self.save_to_disk(notifications);
        }

        item_y += 60.0;

        // تنظیم ۳: اسکن خودکار موزیک
        draw_text("Auto-Scan Music on Startup", content_x + 20.0, item_y + 18.0, 14.0, WHITE);
        let scan_btn = Rect::new(content_x + 480.0, item_y, 110.0, 28.0);
        let scan_hover = scan_btn.contains(mouse_pos.into());
        let scan_col = if scan_hover { Color::new(0.2, 0.4, 0.6, 1.0) } else { Color::new(0.15, 0.25, 0.4, 1.0) };

        draw_rectangle(scan_btn.x, scan_btn.y, scan_btn.w, scan_btn.h, scan_col);
        draw_rectangle_lines(scan_btn.x, scan_btn.y, scan_btn.w, scan_btn.h, 1.0, WHITE);
        let scan_text = if self.config.auto_scan_music { "Active [✔]" } else { "Off [ ]" };
        draw_text(scan_text, scan_btn.x + 20.0, scan_btn.y + 19.0, 13.0, WHITE);

        if scan_hover && clicked {
            self.config.auto_scan_music = !self.config.auto_scan_music;
            self.save_to_disk(notifications);
        }

        item_y += 60.0;

        // تنظیم ۴: نمایش مسیر دانلودها و پورت شبکه
        draw_text(&format!("Default Download Path: {}", self.config.download_path), content_x + 20.0, item_y + 18.0, 13.0, LIGHTGRAY);
        item_y += 40.0;
        draw_text(&format!("Network Discovery Port: {}", self.config.network_port), content_x + 20.0, item_y + 18.0, 13.0, LIGHTGRAY);

        // دکمه بازنشانی به تنظیمات پیش‌فرض
        let reset_btn = Rect::new(content_x + 20.0, box_y + box_h - 50.0, 160.0, 32.0);
        let reset_hover = reset_btn.contains(mouse_pos.into());
        let reset_col = if reset_hover { Color::new(0.5, 0.2, 0.2, 1.0) } else { Color::new(0.35, 0.15, 0.15, 1.0) };

        draw_rectangle(reset_btn.x, reset_btn.y, reset_btn.w, reset_btn.h, reset_col);
        draw_rectangle_lines(reset_btn.x, reset_btn.y, reset_btn.w, reset_btn.h, 1.0, Color::new(0.6, 0.3, 0.3, 1.0));
        draw_text("Reset to Default", reset_btn.x + 22.0, reset_btn.y + 21.0, 13.0, WHITE);

        if reset_hover && clicked {
            self.config = SettingsConfig::default();
            self.save_to_disk(notifications);
            notifications.push("Settings reset to factory defaults.", NotificationKind::Warning);
        }

        // پیام وضعیت پایین صفحه
        draw_text(&self.status_message, content_x, box_y + box_h + 25.0, 13.0, YELLOW);
        draw_text("Press [ESC] to return to Dashboard", content_x, screen_height() - 15.0, 12.0, GRAY);
    }
}

// تابع کمکی برای پارس فایل تنظیمات
fn serde_json_from_str_custom(data: &str) -> Result<SettingsConfig, ()> {
    let mut config = SettingsConfig::default();
    for line in data.lines() {
        if line.contains("theme_dark") {
            config.theme_dark = line.contains("true");
        } else if line.contains("master_volume") {
            if let Some(val_str) = line.split(':').nth(1) {
                if let Ok(v) = val_str.trim().trim_end_matches(',').parse::<f32>() {
                    config.master_volume = v;
                }
            }
        } else if line.contains("auto_scan_music") {
            config.auto_scan_music = line.contains("true");
        } else if line.contains("network_port") {
            if let Some(val_str) = line.split(':').nth(1) {
                if let Ok(v) = val_str.trim().trim_end_matches(',').parse::<u16>() {
                    config.network_port = v;
                }
            }
        }
    }
    Ok(config)
}
// json serialization sync
