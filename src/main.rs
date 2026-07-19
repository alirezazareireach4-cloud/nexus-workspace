use macroquad::prelude::*;

mod filemanager;
mod download;
mod note;
mod network;
mod notifications;
mod settings;
mod music;

use filemanager::FileManagerState;
use download::DownloaderState;
use note::NotesState;
use network::NetworkState;
use notifications::{NotificationCenter, NotificationKind};
use settings::SettingsState;
use music::MusicLibraryState;

#[derive(Clone, Copy, PartialEq)]
enum AppState {
    Dashboard,
    FileManager,
    Downloader,
    Network,
    Notes,
    Notifications,
    Settings,
    MusicLibrary,
}

#[macroquad::main("Nexus Workspace - Pro Edition")]
async fn main() {
    let mut current_state = AppState::Dashboard;
    let mut file_manager = FileManagerState::new();
    let mut downloader = DownloaderState::new();
    let mut notes_app = NotesState::new();
    let mut network_app = NetworkState::new();
    let mut notifications_app = NotificationCenter::new();
    let mut settings_app = SettingsState::new();
    let mut music_app = MusicLibraryState::new();

    notifications_app.push("System Workspace initialized successfully.", NotificationKind::Success);

    loop {
        // اعمال تم تاریک یا روشن بر اساس تنظیمات واقعی ذخیره شده روی دیسک
        let is_dark = settings_app.config.theme_dark;

        let bg_color = if is_dark { Color::new(0.08, 0.06, 0.14, 1.0) } else { Color::new(0.9, 0.9, 0.95, 1.0) };
        let sidebar_bg = if is_dark { Color::new(0.06, 0.04, 0.11, 1.0) } else { Color::new(0.85, 0.85, 0.9, 1.0) };
        let text_main = if is_dark { WHITE } else { BLACK };
        let text_sub = if is_dark { LIGHTGRAY } else { DARKGRAY };
        let line_color = if is_dark { Color::new(0.15, 0.10, 0.25, 1.0) } else { Color::new(0.7, 0.7, 0.8, 1.0) };

        clear_background(bg_color);

        // ==========================================
        // ۱. منوی کناری (Sidebar)
        // ==========================================
        draw_rectangle(0.0, 0.0, 240.0, screen_height(), sidebar_bg);
        draw_line(240.0, 0.0, 240.0, screen_height(), 1.5, line_color);

        draw_text("NEXUS OS", 25.0, 40.0, 20.0, text_main);
        draw_text("Workspace v1.0", 25.0, 60.0, 12.0, SKYBLUE);
        draw_text("Navigation Menu", 25.0, 95.0, 13.0, text_sub);

        let menu_items = vec![
            ("Dashboard", AppState::Dashboard),
            ("File Manager", AppState::FileManager),
            ("Async Downloader", AppState::Downloader),
            ("Network & Devices", AppState::Network),
            ("Notes Editor", AppState::Notes),
            ("Music Library", AppState::MusicLibrary),
            ("History / Logs", AppState::Notifications),
            ("Settings Hub", AppState::Settings),
        ];

        let mouse_pos = mouse_position();
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        for (i, (item_name, target_state)) in menu_items.iter().enumerate() {
            let y = 115.0 + (i as f32 * 36.0);
            let is_selected = current_state == *target_state;

            if is_selected {
                draw_rectangle(15.0, y - 4.0, 210.0, 28.0, Color::new(0.35, 0.20, 0.75, 1.0));
            } else if mouse_pos.0 >= 15.0 && mouse_pos.0 <= 225.0 && mouse_pos.1 >= y - 4.0 && mouse_pos.1 <= y + 24.0 {
                if mouse_clicked {
                    current_state = *target_state;
                    notifications_app.push(&format!("Switched view to {}", item_name), NotificationKind::Info);
                }
            }

            let item_text_color = if is_selected { WHITE } else { text_sub };
            draw_text(item_name, 25.0, y + 15.0, 14.0, item_text_color);
        }

        let user_box_bg = if is_dark { Color::new(0.12, 0.09, 0.22, 1.0) } else { Color::new(0.8, 0.8, 0.85, 1.0) };
        draw_rectangle(15.0, screen_height() - 75.0, 210.0, 60.0, user_box_bg);
        draw_text("Alireza Zarei", 25.0, screen_height() - 48.0, 13.0, text_main);
        draw_text("Computer Engineering", 25.0, screen_height() - 28.0, 11.0, text_sub);

        // ==========================================
        // ۲. هدر بالا (Top Bar)
        // ==========================================
        draw_rectangle(240.0, 0.0, screen_width() - 240.0, 60.0, sidebar_bg);
        draw_line(240.0, 60.0, screen_width(), 60.0, 1.5, line_color);
        draw_text("System Status: [ACTIVE & RUNNING]", 270.0, 36.0, 14.0, GREEN);

        // ==========================================
        // ۳. محتوای صفحات
        // ==========================================
        let content_x = 270.0;

        match current_state {
            AppState::Dashboard => {
                draw_text("Central Control Dashboard", content_x, 95.0, 20.0, text_main);
                draw_text("Overview of all system subsystems and modules:", content_x, 115.0, 13.0, text_sub);

                let dashboard_cards = vec![
                    ("File Manager", "Local Storage", AppState::FileManager, Color::new(0.2, 0.4, 0.8, 1.0)),
                    ("Async Downloader", "Tokio Engine", AppState::Downloader, Color::new(0.1, 0.6, 0.4, 1.0)),
                    ("Network & Devices", "Local Peers Hub", AppState::Network, Color::new(0.2, 0.5, 0.7, 1.0)),
                    ("Notes Editor", "Disk Persistence", AppState::Notes, Color::new(0.8, 0.6, 0.1, 1.0)),
                    ("Music Library", "Media Streamer", AppState::MusicLibrary, Color::new(0.7, 0.2, 0.6, 1.0)),
                    ("History / Logs", "Event Tracking", AppState::Notifications, Color::new(0.8, 0.4, 0.1, 1.0)),
                    ("Settings Hub", "Configurations", AppState::Settings, Color::new(0.5, 0.2, 0.8, 1.0)),
                ];

                let card_w = 210.0;
                let card_h = 100.0;
                let start_y = 140.0;
                let gap_x = 20.0;
                let gap_y = 20.0;

                for (idx, (title, sub, target_st, col)) in dashboard_cards.into_iter().enumerate() {
                    let col_idx = (idx % 3) as f32;
                    let row_idx = (idx / 3) as f32;
                    let cx = content_x + col_idx * (card_w + gap_x);
                    let cy = start_y + row_idx * (card_h + gap_y);

                    let is_hovered = mouse_pos.0 >= cx && mouse_pos.0 <= cx + card_w && mouse_pos.1 >= cy && mouse_pos.1 <= cy + card_h;
                    let bg_col = if is_hovered {
                        Color::new(col.r, col.g, col.b, if is_dark { 0.35 } else { 0.6 })
                    } else {
                        Color::new(col.r, col.g, col.b, if is_dark { 0.18 } else { 0.3 })
                    };

                    draw_rectangle(cx, cy, card_w, card_h, bg_col);
                    draw_rectangle_lines(cx, cy, card_w, card_h, 1.5, col);

                    draw_text(title, cx + 15.0, cy + 25.0, 15.0, text_main);
                    draw_text(sub, cx + 15.0, cy + 50.0, 12.0, text_sub);
                    draw_text("Click to open ->", cx + 15.0, cy + 78.0, 11.0, if is_dark { YELLOW } else { Color::new(0.7, 0.5, 0.0, 1.0) });

                    if is_hovered && mouse_clicked {
                        current_state = target_st;
                        notifications_app.push(&format!("Opened card: {}", title), NotificationKind::Info);
                    }
                }
            }

            AppState::FileManager => {
                file_manager.update_and_render(content_x);
            }

            AppState::Downloader => {
                downloader.update(content_x);
                draw_text("Press [ESC] to return to Dashboard", content_x, screen_height() - 20.0, 12.0, text_sub);
            }

            AppState::Network => {
                draw_text("🌐 Local Network Discovery & Secure File Transfer", content_x, 95.0, 20.0, SKYBLUE);
                draw_text("Discover peers using UDP broadcast and transfer files securely via TCP:", content_x, 120.0, 13.0, text_sub);

                network_app.update();

                let scan_btn = Rect::new(content_x, 145.0, 180.0, 32.0);
                if gui_button_custom(scan_btn, "Scan Network (UDP)", mouse_pos) {
                    network_app.start_discovery();
                    notifications_app.push("Initiated UDP peer discovery scan.", NotificationKind::Info);
                }

                draw_text(&format!("Status: {}", network_app.connection_status), content_x, 200.0, 14.0, GREEN);
                draw_text("Discovered Devices:", content_x, 235.0, 14.0, if is_dark { YELLOW } else { Color::new(0.6, 0.4, 0.0, 1.0) });
                let mut list_y = 255.0;

                if network_app.discovered_peers.is_empty() {
                    draw_text("No devices detected. Click scan to search.", content_x + 10.0, list_y, 12.0, text_sub);
                } else {
                    for ip in network_app.discovered_peers.clone() {
                        let peer_rect = Rect::new(content_x, list_y, 320.0, 28.0);
                        let is_hovered = peer_rect.contains(mouse_pos.into());
                        let bg_col = if is_hovered {
                            Color::new(0.2, 0.4, 0.5, 1.0)
                        } else {
                            if is_dark { Color::new(0.1, 0.15, 0.25, 1.0) } else { Color::new(0.8, 0.85, 0.9, 1.0) }
                        };

                        draw_rectangle(peer_rect.x, peer_rect.y, peer_rect.w, peer_rect.h, bg_col);
                        draw_text(&format!("Peer IP: {}", ip), peer_rect.x + 10.0, peer_rect.y + 19.0, 13.0, text_main);

                        if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                            network_app.peer_ip = ip.clone();
                            notifications_app.push(&format!("Selected peer target IP: {}", ip), NotificationKind::Info);
                        }

                        list_y += 35.0;
                    }
                }

                let transfer_box_y = 420.0;
                let transfer_box_w = 650.0;
                let t_box_bg = if is_dark { Color::new(0.06, 0.08, 0.14, 1.0) } else { Color::new(0.95, 0.95, 0.95, 1.0) };
                draw_rectangle(content_x, transfer_box_y, transfer_box_w, 140.0, t_box_bg);
                draw_rectangle_lines(content_x, transfer_box_y, transfer_box_w, 140.0, 1.0, SKYBLUE);

                draw_text(&format!("Target Selected IP: {}", network_app.peer_ip), content_x + 15.0, transfer_box_y + 25.0, 13.0, text_main);
                draw_text(&format!("Transfer Log: {}", network_app.transfer_status), content_x + 15.0, transfer_box_y + 50.0, 13.0, text_sub);

                let prog_bg = if is_dark { Color::new(0.1, 0.1, 0.2, 1.0) } else { Color::new(0.8, 0.8, 0.85, 1.0) };
                draw_rectangle(content_x + 15.0, transfer_box_y + 70.0, transfer_box_w - 30.0, 12.0, prog_bg);
                let prog_w = (network_app.progress / 100.0) * (transfer_box_w - 30.0);
                draw_rectangle(content_x + 15.0, transfer_box_y + 70.0, prog_w, 12.0, GREEN);

                let send_btn = Rect::new(content_x + 15.0, transfer_box_y + 95.0, 200.0, 30.0);
                if gui_button_custom(send_btn, "Select File & Send", mouse_pos) {
                    let target = network_app.peer_ip.clone();
                    network_app.select_and_send_file(&target);
                    notifications_app.push(&format!("Opened file dialog for target {}", target), NotificationKind::Success);
                }

                draw_text("Press [ESC] to return to Dashboard", content_x, screen_height() - 20.0, 12.0, text_sub);
            }

            AppState::Notes => {
                notes_app.update(content_x);
                if is_key_pressed(KeyCode::F2) {
                    notes_app.save();
                    notifications_app.push("Note successfully saved to disk.", NotificationKind::Success);
                }
                draw_text("Press [F2] to Save Note | [ESC] Dashboard", content_x, screen_height() - 20.0, 12.0, text_sub);
            }

            AppState::MusicLibrary => {
                music_app.update(content_x);
            }

            AppState::Notifications => {
                notifications_app.update(content_x);
            }

            AppState::Settings => {
                // پاس دادن notifications برای ذخیره سازی
                settings_app.update(content_x, &mut notifications_app);
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            current_state = AppState::Dashboard;
        }

        next_frame().await;
    }
}

fn gui_button_custom(rect: Rect, text: &str, mouse: (f32, f32)) -> bool {
    let hovered = rect.contains(mouse.into());
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let col = if hovered { Color::new(0.3, 0.5, 0.4, 1.0) } else { Color::new(0.2, 0.3, 0.25, 1.0) };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, WHITE);

    let text_x = rect.x + (rect.w / 2.0) - (text.len() as f32 * 3.2);
    let text_y = rect.y + 20.0;
    draw_text(text, text_x, text_y, 13.0, WHITE);

    clicked
}