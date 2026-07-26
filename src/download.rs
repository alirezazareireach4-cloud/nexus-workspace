use macroquad::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
pub enum DownloadCommand {
    Pause,
    Resume,
    Cancel,
}

pub struct DownloadItem {
    pub id: usize,
    pub url: String,
    pub filename: String,
    pub status: String,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub is_downloading: bool,
    pub is_paused: bool,
    pub receiver: Receiver<DownloadUpdate>,
    pub cmd_sender: Option<Sender<DownloadCommand>>,
}

pub enum DownloadUpdate {
    Progress(f32, u64, u64),
    Completed(String),
    Paused,
    Failed(String),
}

pub struct DownloaderState {
    pub download_url: String,
    pub downloads: Vec<DownloadItem>,
    next_id: usize,
}

impl DownloaderState {
    pub fn new() -> Self {
        Self {
            download_url: "".to_string(),
            downloads: Vec::new(),
            next_id: 1,
        }
    }

    pub fn update(&mut self, content_x: f32) {

        for item in &mut self.downloads {
            while let Ok(update) = item.receiver.try_recv() {
                match update {
                    DownloadUpdate::Progress(ratio, downloaded, total) => {
                        item.progress = ratio;
                        item.downloaded_bytes = downloaded;
                        item.total_bytes = total;
                        if total > 0 {
                            item.status = format!("Downloading... {} / {} KB", downloaded / 1024, total / 1024);
                        } else {
                            item.status = format!("Downloading... {} KB downloaded", downloaded / 1024);
                        }
                    }
                    DownloadUpdate::Completed(filename) => {
                        item.is_downloading = false;
                        item.is_paused = false;
                        item.progress = 1.0;
                        item.status = format!("Completed: '{}'", filename);
                        item.cmd_sender = None;
                    }
                    DownloadUpdate::Paused => {
                        item.is_paused = true;
                        item.status = "Paused.".to_string();
                    }
                    DownloadUpdate::Failed(err) => {
                        item.is_downloading = false;
                        item.is_paused = false;
                        item.status = format!("Error: {}", err);
                        item.cmd_sender = None;
                    }
                }
            }
        }

        let mouse_pos = mouse_position();
        let box_w = 720.0;
        let box_h = 420.0;
        let box_y = 180.0;


        draw_text("📥 Advanced Download Manager", content_x, 80.0, 22.0, BLUE);
        draw_text("Enter Download URL:", content_x, 115.0, 14.0, LIGHTGRAY);


        let input_w = box_w - 185.0;
        let input_rect = Rect::new(content_x, 130.0, input_w, 32.0);
        draw_rectangle(input_rect.x, input_rect.y, input_rect.w, input_rect.h, Color::new(0.1, 0.12, 0.18, 1.0));
        draw_rectangle_lines(input_rect.x, input_rect.y, input_rect.w, input_rect.h, 1.0, SKYBLUE);

        let display_text = if self.download_url.is_empty() {
            "Paste or type link here...".to_string()
        } else {
            format!("{}_", self.download_url)
        };
        let text_color = if self.download_url.is_empty() { GRAY } else { WHITE };
        draw_text(&display_text, input_rect.x + 10.0, input_rect.y + 21.0, 14.0, text_color);


        let chars = get_char_pressed();
        for c in chars {
            if c == '\u{8}' {
                self.download_url.pop();
            } else if !c.is_control() && self.mode_is_typing(input_rect, mouse_pos) {
                self.download_url.push(c);
            }
        }


        if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
            if is_key_pressed(KeyCode::V) {
                if let Some(clip) = macroquad::window::miniquad::window::clipboard_get() {
                    if !clip.is_empty() {
                        self.download_url = clip.trim().to_string();
                    }
                }
            }
        }


        let paste_btn_rect = Rect::new(content_x + input_w + 5.0, 130.0, 75.0, 32.0);
        if gui_button(paste_btn_rect, "Paste", mouse_pos) {
            if let Some(clip) = macroquad::window::miniquad::window::clipboard_get() {
                if !clip.is_empty() {
                    self.download_url = clip.trim().to_string();
                }
            }
        }


        let add_btn_rect = Rect::new(content_x + box_w - 100.0, 130.0, 100.0, 32.0);
        if gui_button(add_btn_rect, "+ Add", mouse_pos) && !self.download_url.is_empty() {
            self.start_new_download(self.download_url.clone());
            self.download_url.clear();
        }


        draw_rectangle(content_x, box_y, box_w, box_h, Color::new(0.05, 0.07, 0.12, 1.0));
        draw_rectangle_lines(content_x, box_y, box_w, box_h, 1.5, BLUE);

        draw_text("Active & Paused Downloads:", content_x + 15.0, box_y + 30.0, 16.0, WHITE);


        let mut item_y = box_y + 55.0;
        let mut action_to_remove = None;

        for (index, item) in self.downloads.iter_mut().enumerate() {
            let row_rect = Rect::new(content_x + 15.0, item_y, box_w - 30.0, 65.0);
            draw_rectangle(row_rect.x, row_rect.y, row_rect.w, row_rect.h, Color::new(0.08, 0.1, 0.16, 1.0));
            draw_rectangle_lines(row_rect.x, row_rect.y, row_rect.w, row_rect.h, 1.0, Color::new(0.2, 0.3, 0.4, 1.0));


            draw_text(&format!("[{}] {}", item.id, item.filename), row_rect.x + 10.0, row_rect.y + 20.0, 14.0, WHITE);
            draw_text(&item.status, row_rect.x + 10.0, row_rect.y + 45.0, 12.0, YELLOW);


            let bar_x = row_rect.x + 240.0;
            let bar_y = row_rect.y + 15.0;
            let bar_w = 180.0;
            let bar_h = 16.0;

            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.15, 0.18, 0.25, 1.0));
            if item.progress > 0.0 {
                draw_rectangle(bar_x, bar_y, bar_w * item.progress, bar_h, Color::new(0.2, 0.6, 0.3, 0.9));
            }
            draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, LIGHTGRAY);
            draw_text(&format!("{}%", (item.progress * 100.0) as i32), bar_x + bar_w + 8.0, bar_y + 13.0, 12.0, WHITE);


            let btn_x = row_rect.x + row_rect.w - 225.0;
            let btn_y = row_rect.y + 15.0;

            if item.is_downloading {
                if !item.is_paused {
                    if gui_button(Rect::new(btn_x, btn_y, 65.0, 28.0), "Pause", mouse_pos) {
                        if let Some(ref tx) = item.cmd_sender { let _ = tx.send(DownloadCommand::Pause); }
                    }
                } else {
                    if gui_button(Rect::new(btn_x, btn_y, 65.0, 28.0), "Resume", mouse_pos) {
                        if let Some(ref tx) = item.cmd_sender { let _ = tx.send(DownloadCommand::Resume); }
                    }
                }
                if gui_button(Rect::new(btn_x + 70.0, btn_y, 65.0, 28.0), "Cancel", mouse_pos) {
                    if let Some(ref tx) = item.cmd_sender { let _ = tx.send(DownloadCommand::Cancel); }
                    action_to_remove = Some(index);
                }
            } else {
                if gui_button(Rect::new(btn_x, btn_y, 135.0, 28.0), "Remove", mouse_pos) {
                    action_to_remove = Some(index);
                }
            }

            item_y += 75.0;
            if item_y > box_y + box_h - 60.0 { break; }
        }

        if let Some(idx) = action_to_remove {
            self.downloads.remove(idx);
        }
    }

    fn start_new_download(&mut self, url: String) {
        let (tx, rx) = channel();
        let (cmd_tx, cmd_rx) = channel();

        let current_id = self.next_id;
        let clean_name = {
            let path_part = url.split('?').next().unwrap_or(&url);
            let raw_name = path_part.split('/').last().unwrap_or("");
            if raw_name.is_empty() || !raw_name.contains('.') {
                format!("file_{}.bin", current_id)
            } else {
                raw_name.to_string()
            }
        };

        let item = DownloadItem {
            id: current_id,
            url: url.clone(),
            filename: clean_name.clone(),
            status: "Starting...".to_string(),
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            is_downloading: true,
            is_paused: false,
            receiver: rx,
            cmd_sender: Some(cmd_tx),
        };

        self.downloads.push(item);
        self.next_id += 1;

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                    return;
                }
            };

            rt.block_on(async move {
                let client = match reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::limited(10))
                    .build()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                        return;
                    }
                };


                let mut downloaded_bytes = 0u64;
                if let Ok(metadata) = std::fs::metadata(&clean_name) {
                    downloaded_bytes = metadata.len();
                }


                let head_resp = client.head(&url).send().await;
                if let Ok(hr) = head_resp {
                    if let Some(total_len) = hr.content_length() {

                        if downloaded_bytes >= total_len && total_len > 0 {
                            let _ = std::fs::remove_file(&clean_name);
                            downloaded_bytes = 0;
                        }
                    }
                }

                let mut req = client.get(&url);
                if downloaded_bytes > 0 {
                    req = req.header("Range", format!("bytes={}-", downloaded_bytes));
                }

                let resp = match req.send().await {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                        return;
                    }
                };

                if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {

                    if resp.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                        let _ = std::fs::remove_file(&clean_name);
                        let _ = tx.send(DownloadUpdate::Failed("Range error fixed, please restart download.".to_string()));
                    } else {
                        let _ = tx.send(DownloadUpdate::Failed(format!("HTTP Error: {}", resp.status())));
                    }
                    return;
                }

                let content_len = resp.content_length().unwrap_or(0);
                let total_size = content_len + downloaded_bytes;

                let mut file = match OpenOptions::new().create(true).append(true).open(&clean_name) {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                        return;
                    }
                };

                let mut stream = resp;
                let mut paused = false;

                loop {
                    if let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            DownloadCommand::Pause => {
                                paused = true;
                                let _ = tx.send(DownloadUpdate::Paused);
                            }
                            DownloadCommand::Resume => {
                                paused = false;
                            }
                            DownloadCommand::Cancel => {
                                let _ = tx.send(DownloadUpdate::Failed("Cancelled.".to_string()));
                                return;
                            }
                        }
                    }

                    if paused {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }

                    match stream.chunk().await {
                        Ok(Some(chunk)) => {
                            if let Err(e) = file.write_all(&chunk) {
                                let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                                return;
                            }
                            downloaded_bytes += chunk.len() as u64;

                            let ratio = if total_size > 0 {
                                (downloaded_bytes as f32 / total_size as f32).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };

                            let _ = tx.send(DownloadUpdate::Progress(ratio, downloaded_bytes, total_size));
                        }
                        Ok(None) => {
                            let _ = tx.send(DownloadUpdate::Completed(clean_name));
                            break;
                        }
                        Err(e) => {
                            let _ = tx.send(DownloadUpdate::Failed(e.to_string()));
                            break;
                        }
                    }
                }
            });
        });
    }

    fn mode_is_typing(&self, rect: Rect, mouse: (f32, f32)) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && rect.contains(mouse.into())
    }
}

fn gui_button(rect: Rect, text: &str, mouse: (f32, f32)) -> bool {
    let hovered = rect.contains(mouse.into());
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let col = if hovered { Color::new(0.35, 0.45, 0.65, 1.0) } else { Color::new(0.2, 0.25, 0.35, 1.0) };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, WHITE);

    let text_x = rect.x + (rect.w / 2.0) - (text.len() as f32 * 3.5);
    let text_y = rect.y + 18.0;
    draw_text(text, text_x, text_y, 13.0, WHITE);

    clicked
}
// download task sync
