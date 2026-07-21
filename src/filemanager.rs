use macroquad::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use arboard::Clipboard;

pub struct FileManagerState {
    pub current_path: PathBuf,
    pub entries: Vec<(String, bool, u64)>,
    pub selected_index: usize,
    pub status: String,
    pub search_query: String,
    pub mode: FileMgrMode,
    pub input_buffer: String,
    pub scroll_offset: f32,
    pub new_file_type: Option<NewFileType>,
    pub pending_send_item: Option<PathBuf>,
    pub is_copy_action: bool,
    pub last_click_time: f64,
    pub last_clicked_index: usize,
}

#[derive(PartialEq, Clone, Copy)]
pub enum NewFileType {
    Folder,
    TextFile,
    ZipFile,
}

#[derive(PartialEq)]
pub enum FileMgrMode {
    Normal,
    PromptingNewType,
    CreatingNewName,
    Renaming,
    Searching,
    PromptingSendMode,
    EnteringSendTarget,
    ConfirmingDelete,
}

impl FileManagerState {
    pub fn new() -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let entries = Self::load_entries(&current_path);
        Self {
            current_path,
            entries,
            selected_index: 0,
            status: "Ready. Use toolbar buttons or keyboard shortcuts.".to_string(),
            search_query: String::new(),
            mode: FileMgrMode::Normal,
            input_buffer: String::new(),
            scroll_offset: 0.0,
            new_file_type: None,
            pending_send_item: None,
            is_copy_action: true,
            last_click_time: 0.0,
            last_clicked_index: 999999,
        }
    }

    pub fn load_entries(path: &Path) -> Vec<(String, bool, u64)> {
        let mut result = Vec::new();
        if let Ok(read_dir) = fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') { continue; }
                let metadata = entry.metadata();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| if m.is_file() { m.len() } else { 0 }).unwrap_or(0);
                result.push((name, is_dir, size));
            }
        }
        result.sort_by(|a, b| {
            if a.1 == b.1 { a.0.to_lowercase().cmp(&b.0.to_lowercase()) }
            else { b.1.cmp(&a.1) }
        });
        result
    }

    pub fn navigate_into(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.entries = Self::load_entries(&self.current_path);
            self.selected_index = 0;
            self.scroll_offset = 0.0;
            self.search_query.clear();
            self.status = format!("Opened: {:?}", self.current_path);
        }
    }

    pub fn open_selected(&mut self) {
        let filtered = self.get_filtered_entries();
        if filtered.is_empty() { return; }
        if let Some((name, is_dir, _)) = filtered.get(self.selected_index).cloned() {
            let target_path = self.current_path.join(name);
            if is_dir {
                self.navigate_into(target_path);
            } else {
                self.status = format!("File selected: {:?}", target_path);
            }
        }
    }

    pub fn delete_selected_confirmed(&mut self) {
        let filtered = self.get_filtered_entries();
        if filtered.is_empty() { return; }
        let (name, is_dir, _) = filtered[self.selected_index].clone();
        let target_path = self.current_path.join(name);

        let res = if is_dir { fs::remove_dir_all(&target_path) } else { fs::remove_file(&target_path) };
        match res {
            Ok(_) => {
                self.status = format!("Deleted: {}", target_path.display());
                self.entries = Self::load_entries(&self.current_path);
                if self.selected_index > 0 && self.selected_index >= self.get_filtered_entries().len() {
                    self.selected_index = self.get_filtered_entries().len().saturating_sub(1);
                }
            }
            Err(e) => self.status = format!("Error: {}", e),
        }
        self.mode = FileMgrMode::Normal;
    }

    pub fn execute_send(&mut self, target_dir_str: &str) {
        if let Some(ref source_path) = self.pending_send_item.clone() {
            let target_base = PathBuf::from(target_dir_str);
            if !target_base.exists() || !target_base.is_dir() {
                self.status = "Error: Target directory does not exist or is invalid!".to_string();
                return;
            }

            if let Some(file_name) = source_path.file_name() {
                let dest_path = target_base.join(file_name);
                if source_path == &dest_path {
                    self.status = "Error: Cannot send into the same location!".to_string();
                    return;
                }

                let res = if self.is_copy_action {
                    if source_path.is_dir() {
                        self.copy_dir_all(source_path, &dest_path)
                    } else {
                        fs::copy(source_path, &dest_path).map(|_| ())
                    }
                } else {
                    fs::rename(source_path, &dest_path)
                };

                match res {
                    Ok(_) => {
                        let action_name = if self.is_copy_action { "Copied" } else { "Moved (Cut)" };
                        self.status = format!("Successfully {} to: {:?}", action_name, dest_path);
                        self.entries = Self::load_entries(&self.current_path);
                        self.mode = FileMgrMode::Normal;
                        self.pending_send_item = None;
                        self.input_buffer.clear();
                    }
                    Err(e) => {
                        self.status = format!("Send error: {}", e);
                    }
                }
            }
        }
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dest_path = dst.join(entry.file_name());
            if ty.is_dir() {
                self.copy_dir_all(&entry.path(), &dest_path)?;
            } else {
                fs::copy(entry.path(), dest_path)?;
            }
        }
        Ok(())
    }

    pub fn rename_selected(&mut self, new_name: &str) {
        if new_name.is_empty() { return; }
        let filtered = self.get_filtered_entries();
        if filtered.is_empty() { return; }
        let (old_name, _, _) = filtered[self.selected_index].clone();
        let old_path = self.current_path.join(old_name);
        let new_path = self.current_path.join(new_name);

        match fs::rename(&old_path, &new_path) {
            Ok(_) => {
                self.status = format!("Renamed to: {}", new_name);
                self.entries = Self::load_entries(&self.current_path);
                self.mode = FileMgrMode::Normal;
                self.input_buffer.clear();
            }
            Err(e) => self.status = format!("Rename error: {}", e),
        }
    }

    pub fn create_new_item(&mut self, name: &str) {
        if name.is_empty() { return; }
        let target_path = self.current_path.join(name);

        let res = match self.new_file_type {
            Some(NewFileType::Folder) => fs::create_dir(&target_path).map(|_| ()),
            Some(NewFileType::TextFile) => {
                let mut file = fs::File::create(&target_path);
                if let Ok(ref mut f) = file {
                    let _ = f.write_all(b"");
                }
                file.map(|_| ())
            }
            Some(NewFileType::ZipFile) => {
                let mut file = fs::File::create(&target_path);
                if let Ok(ref mut f) = file {
                    let _ = f.write_all(b"PK\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
                }
                file.map(|_| ())
            }
            None => Ok(()),
        };

        match res {
            Ok(_) => {
                self.status = format!("Created: {}", name);
                self.entries = Self::load_entries(&self.current_path);
                self.mode = FileMgrMode::Normal;
                self.input_buffer.clear();
                self.new_file_type = None;
            }
            Err(e) => self.status = format!("Creation error: {}", e),
        }
    }

    pub fn get_filtered_entries(&self) -> Vec<(String, bool, u64)> {
        if self.search_query.is_empty() {
            self.entries.clone()
        } else {
            let query = self.search_query.to_lowercase();
            self.entries
                .iter()
                .filter(|(name, _, _)| name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    }

    pub fn update_and_render(&mut self, content_x: f32) {
        let filtered = self.get_filtered_entries();

        if filtered.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= filtered.len() {
            self.selected_index = filtered.len() - 1;
        }

        if self.mode == FileMgrMode::Normal {
            if is_key_pressed(KeyCode::Down) {
                if !filtered.is_empty() && self.selected_index < filtered.len() - 1 {
                    self.selected_index += 1;
                    let selected_y = (self.selected_index as f32) * 30.0;
                    if selected_y - self.scroll_offset > 370.0 - 40.0 {
                        self.scroll_offset = selected_y - (370.0 - 40.0);
                    }
                }
            }
            if is_key_pressed(KeyCode::Up) {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    let selected_y = (self.selected_index as f32) * 30.0;
                    if selected_y < self.scroll_offset {
                        self.scroll_offset = selected_y;
                    }
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                self.open_selected();
            }
            if is_key_pressed(KeyCode::Backspace) {
                if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
                    self.navigate_into(parent);
                }
            }
            if is_key_pressed(KeyCode::F2) && !filtered.is_empty() {
                self.mode = FileMgrMode::Renaming;
                self.input_buffer = filtered[self.selected_index].0.clone();
            }
            if is_key_pressed(KeyCode::F3) {
                self.mode = FileMgrMode::Searching;
                self.input_buffer = self.search_query.clone();
            }
        } else {
            let chars = get_char_pressed();
            for c in chars {
                if c == '\u{8}' {
                    self.input_buffer.pop();
                } else if c == '\r' || c == '\n' {
                    let val = self.input_buffer.clone();
                    match self.mode {
                        FileMgrMode::CreatingNewName => self.create_new_item(&val),
                        FileMgrMode::Renaming => self.rename_selected(&val),
                        FileMgrMode::Searching => {
                            self.search_query = val;
                            self.mode = FileMgrMode::Normal;
                        }
                        FileMgrMode::EnteringSendTarget => {
                            self.execute_send(&val);
                        }
                        _ => {}
                    }
                } else if !c.is_control() {
                    self.input_buffer.push(c);
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                self.mode = FileMgrMode::Normal;
                self.input_buffer.clear();
            }
        }

        let mouse_pos = mouse_position();
        let item_h = 30.0;
        let box_y = 210.0;
        let box_w = 670.0;
        let box_h = 340.0;

        let total_content_height = (filtered.len() as f32) * item_h;
        let max_scroll = (total_content_height - box_h + 20.0).max(0.0);

        let wheel = mouse_wheel().1;
        if wheel != 0.0 && self.mode == FileMgrMode::Normal {
            self.scroll_offset = (self.scroll_offset - wheel * 30.0).clamp(0.0, max_scroll);
        }

        draw_text("📁 Advanced File Manager", content_x, 90.0, 20.0, BLUE);
        draw_text(&format!("Path: {:?}", self.current_path), content_x, 115.0, 13.0, LIGHTGRAY);

        if !self.search_query.is_empty() {
            draw_text(&format!("Search Filter: '{}' (Press Esc to clear)", self.search_query), content_x, 195.0, 13.0, YELLOW);
        }

        let btn_y = 135.0;
        if gui_button(Rect::new(content_x, btn_y, 60.0, 30.0), ".. Up", mouse_pos) {
            if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
                self.navigate_into(parent);
            }
        }
        if gui_button(Rect::new(content_x + 65.0, btn_y, 75.0, 30.0), "+ New", mouse_pos) {
            self.mode = FileMgrMode::PromptingNewType;
        }
        if gui_button(Rect::new(content_x + 145.0, btn_y, 80.0, 30.0), "Rename", mouse_pos) && !filtered.is_empty() {
            self.mode = FileMgrMode::Renaming;
            self.input_buffer = filtered[self.selected_index].0.clone();
        }
        if gui_button(Rect::new(content_x + 230.0, btn_y, 75.0, 30.0), "Send", mouse_pos) && !filtered.is_empty() {
            let (name, _, _) = filtered[self.selected_index].clone();
            self.pending_send_item = Some(self.current_path.join(name));
            self.mode = FileMgrMode::PromptingSendMode;
        }
        if gui_button(Rect::new(content_x + 310.0, btn_y, 80.0, 30.0), "Search", mouse_pos) {
            self.mode = FileMgrMode::Searching;
            self.input_buffer = self.search_query.clone();
        }
        if gui_button(Rect::new(content_x + 395.0, btn_y, 80.0, 30.0), "Delete", mouse_pos) && !filtered.is_empty() {
            self.mode = FileMgrMode::ConfirmingDelete;
        }

        if self.mode == FileMgrMode::PromptingNewType {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.12, 0.15, 0.25, 0.95));
            draw_text("Select Type:", content_x + 10.0, 195.0, 13.0, WHITE);
            if gui_button(Rect::new(content_x + 100.0, 178.0, 80.0, 26.0), "Folder", mouse_pos) {
                self.new_file_type = Some(NewFileType::Folder);
                self.mode = FileMgrMode::CreatingNewName;
                self.input_buffer.clear();
            }
            if gui_button(Rect::new(content_x + 190.0, 178.0, 95.0, 26.0), "Text (.txt)", mouse_pos) {
                self.new_file_type = Some(NewFileType::TextFile);
                self.mode = FileMgrMode::CreatingNewName;
                self.input_buffer.clear();
            }
            if gui_button(Rect::new(content_x + 295.0, 178.0, 95.0, 26.0), "Zip (.zip)", mouse_pos) {
                self.new_file_type = Some(NewFileType::ZipFile);
                self.mode = FileMgrMode::CreatingNewName;
                self.input_buffer.clear();
            }
            if gui_button(Rect::new(content_x + 400.0, 178.0, 70.0, 26.0), "Cancel", mouse_pos) {
                self.mode = FileMgrMode::Normal;
            }
        }

        if self.mode == FileMgrMode::ConfirmingDelete {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.3, 0.1, 0.1, 0.95));
            draw_text("Are you sure to delete?", content_x + 10.0, 195.0, 13.0, WHITE);
            if gui_button(Rect::new(content_x + 160.0, 178.0, 60.0, 26.0), "Yes", mouse_pos) {
                self.delete_selected_confirmed();
            }
            if gui_button(Rect::new(content_x + 225.0, 178.0, 60.0, 26.0), "No", mouse_pos) {
                self.mode = FileMgrMode::Normal;
            }
        }

        if self.mode == FileMgrMode::PromptingSendMode {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.15, 0.2, 0.25, 0.95));
            draw_text("Send Mode:", content_x + 10.0, 195.0, 13.0, WHITE);
            if gui_button(Rect::new(content_x + 100.0, 178.0, 90.0, 26.0), "1. Copy", mouse_pos) {
                self.is_copy_action = true;
                self.mode = FileMgrMode::EnteringSendTarget;
                self.input_buffer.clear();
            }
            if gui_button(Rect::new(content_x + 195.0, 178.0, 90.0, 26.0), "2. Cut", mouse_pos) {
                self.is_copy_action = false;
                self.mode = FileMgrMode::EnteringSendTarget;
                self.input_buffer.clear();
            }
            if gui_button(Rect::new(content_x + 290.0, 178.0, 70.0, 26.0), "Cancel", mouse_pos) {
                self.mode = FileMgrMode::Normal;
            }
        }

        if self.mode == FileMgrMode::EnteringSendTarget {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.1, 0.2, 0.2, 0.95));
            let mode_str = if self.is_copy_action { "Copy To" } else { "Cut To" };
            draw_text(&format!("{}: {}_", mode_str, self.input_buffer), content_x + 10.0, 195.0, 13.0, SKYBLUE);

            if gui_button(Rect::new(content_x + 480.0, 178.0, 70.0, 26.0), "Paste", mouse_pos) {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(contents) = clipboard.get_text() {
                        self.input_buffer = contents.trim().to_string();
                        self.status = "Pasted from system clipboard successfully.".to_string();
                    } else {
                        self.status = "Clipboard is empty or contains no text.".to_string();
                    }
                } else {
                    self.status = "Could not access system clipboard.".to_string();
                }
            }
        }

        if self.mode == FileMgrMode::CreatingNewName {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.1, 0.2, 0.15, 0.95));
            draw_text(&format!("Enter Name: {}_", self.input_buffer), content_x + 10.0, 195.0, 14.0, GREEN);
        }

        if self.mode == FileMgrMode::Renaming {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.2, 0.15, 0.1, 0.95));
            draw_text(&format!("New Name: {}_", self.input_buffer), content_x + 10.0, 195.0, 14.0, YELLOW);
        }

        if self.mode == FileMgrMode::Searching {
            draw_rectangle(content_x, 175.0, box_w, 32.0, Color::new(0.15, 0.15, 0.25, 0.95));
            draw_text(&format!("Search Query (Enter): {}_", self.input_buffer), content_x + 10.0, 195.0, 14.0, SKYBLUE);
        }

        draw_rectangle(content_x, box_y, box_w, box_h, Color::new(0.05, 0.07, 0.12, 1.0));

        let mut y_pos = box_y + 5.0 - self.scroll_offset;
        for (i, (name, is_dir, size)) in filtered.iter().enumerate() {
            let item_rect = Rect::new(content_x + 5.0, y_pos, box_w - 10.0, item_h - 4.0);
            let is_visible = item_rect.y >= box_y && item_rect.y + item_rect.h <= box_y + box_h;

            if is_visible {
                let is_inside_box = mouse_pos.0 >= content_x && mouse_pos.0 <= content_x + box_w
                    && mouse_pos.1 >= box_y && mouse_pos.1 <= box_y + box_h;

                let is_hovered = is_inside_box && item_rect.contains(mouse_pos.into());

                if is_hovered && is_mouse_button_pressed(MouseButton::Left) && self.mode == FileMgrMode::Normal {
                    let now = get_time();
                    let time_diff = now - self.last_click_time;

                    if self.last_clicked_index == i && time_diff < 0.4 {
                        self.open_selected();
                        self.last_click_time = 0.0;
                    } else {
                        self.selected_index = i;
                        self.last_click_time = now;
                        self.last_clicked_index = i;
                    }
                }

                let bg = if i == self.selected_index {
                    Color::new(0.2, 0.4, 0.7, 0.8)
                } else if is_hovered {
                    Color::new(0.25, 0.35, 0.5, 0.6)
                } else {
                    Color::new(0.0, 0.0, 0.0, 0.0)
                };

                if bg.a > 0.0 {
                    draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg);
                }

                let icon = if *is_dir { "📁" } else { "📄" };
                draw_text(&format!("{}  {} ({} bytes)", icon, name, size), item_rect.x + 10.0, item_rect.y + 20.0, 14.0, WHITE);
            }

            y_pos += item_h;
        }

        draw_rectangle_lines(content_x, box_y, box_w, box_h, 1.5, BLUE);
        draw_rectangle(content_x, 560.0, box_w, 25.0, Color::new(0.08, 0.08, 0.12, 1.0));
        draw_text(&self.status, content_x + 10.0, 577.0, 13.0, YELLOW);
    }
}

fn gui_button(rect: Rect, text: &str, mouse: (f32, f32)) -> bool {
    let hovered = rect.contains(mouse.into());
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let col = if hovered { Color::new(0.3, 0.4, 0.6, 1.0) } else { Color::new(0.2, 0.2, 0.3, 1.0) };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, WHITE);
    draw_text(text, rect.x + 10.0, rect.y + 21.0, 14.0, WHITE);

    clicked
}