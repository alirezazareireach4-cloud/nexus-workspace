use macroquad::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct NoteItem {
    pub id: usize,
    pub title: String,
    pub content: String,
}

#[derive(PartialEq)]
enum FocusedField {
    Title,
    Content,
}

pub struct NotesState {
    pub notes: Vec<NoteItem>,
    pub selected_id: Option<usize>,
    pub next_id: usize,
    pub status_message: String,
    focused_field: FocusedField,
}

const STORAGE_DIR: &str = "notes";

impl NotesState {
    pub fn new() -> Self {
        let mut state = Self {
            notes: Vec::new(),
            selected_id: None,
            next_id: 1,
            status_message: "Ready. Loaded from 'notes' folder.".to_string(),
            focused_field: FocusedField::Content,
        };

        if !Path::new(STORAGE_DIR).exists() {
            let _ = fs::create_dir_all(STORAGE_DIR);
        }

        if let Ok(entries) = fs::read_dir(STORAGE_DIR) {
            let mut max_id = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "txt" {
                            if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                                let title = file_name.to_string();
                                let content = fs::read_to_string(&path).unwrap_or_default();
                                max_id += 1;
                                state.notes.push(NoteItem {
                                    id: max_id,
                                    title,
                                    content,
                                });
                            }
                        }
                    }
                }
            }
            state.next_id = max_id;
        }

        if !state.notes.is_empty() {
            state.selected_id = Some(state.notes[0].id);
        }

        state
    }

    pub fn save_to_disk(&mut self) {
        if !Path::new(STORAGE_DIR).exists() {
            let _ = fs::create_dir_all(STORAGE_DIR);
        }

        for note in &self.notes {
            let safe_title: String = note.title
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect();

            let file_name = if safe_title.trim().is_empty() {
                format!("note_{}", note.id)
            } else {
                safe_title.trim().to_string()
            };

            let file_path = format!("{}/{}.txt", STORAGE_DIR, file_name);
            let _ = fs::write(file_path, &note.content);
        }

        self.status_message = "Notes saved to 'notes/'!".to_string();
    }

    pub fn save(&mut self) {
        self.save_to_disk();
    }

    pub fn update(&mut self, content_x: f32) {
        let mouse_pos = mouse_position();
        let box_w = 720.0;
        let box_h = 420.0;
        let box_y = 180.0;

        draw_text("📝 Professional Notes & Workspace", content_x, 80.0, 22.0, GREEN);
        draw_text("Manage your daily thoughts, snippets, and tasks (Auto-saved):", content_x, 115.0, 14.0, LIGHTGRAY);

        draw_rectangle(content_x, box_y, box_w, box_h, Color::new(0.05, 0.07, 0.12, 1.0));
        draw_rectangle_lines(content_x, box_y, box_w, box_h, 1.5, GREEN);

        let sidebar_w = 220.0;
        draw_rectangle(content_x, box_y, sidebar_w, box_h, Color::new(0.07, 0.09, 0.15, 1.0));
        draw_line(content_x + sidebar_w, box_y, content_x + sidebar_w, box_y + box_h, 1.0, Color::new(0.2, 0.3, 0.4, 1.0));

        draw_text("Your Notes", content_x + 15.0, box_y + 25.0, 15.0, WHITE);

        let new_btn_rect = Rect::new(content_x + 15.0, box_y + 35.0, sidebar_w - 30.0, 28.0);
        if gui_button(new_btn_rect, "+ New Note", mouse_pos) {
            self.next_id += 1;
            let new_id = self.next_id;
            let new_title = format!("Note {}", new_id);

            let file_path = format!("{}/{}.txt", STORAGE_DIR, new_title);
            let _ = fs::write(&file_path, "");

            self.notes.push(NoteItem {
                id: new_id,
                title: new_title,
                content: "".to_string(),
            });
            self.selected_id = Some(new_id);
            self.focused_field = FocusedField::Title;
        }

        let mut list_y = box_y + 75.0;
        let mut id_to_select = None;

        for note in &self.notes {
            let item_rect = Rect::new(content_x + 10.0, list_y, sidebar_w - 20.0, 32.0);
            let is_selected = self.selected_id == Some(note.id);

            let bg_col = if is_selected {
                Color::new(0.2, 0.4, 0.3, 0.8)
            } else if item_rect.contains(mouse_pos.into()) {
                Color::new(0.15, 0.2, 0.25, 1.0)
            } else {
                Color::new(0.09, 0.12, 0.18, 1.0)
            };

            draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg_col);
            draw_text(&note.title, item_rect.x + 10.0, item_rect.y + 21.0, 13.0, WHITE);

            if is_mouse_button_pressed(MouseButton::Left) && item_rect.contains(mouse_pos.into()) {
                id_to_select = Some(note.id);
            }

            list_y += 38.0;
            if list_y > box_y + box_h - 40.0 { break; }
        }

        if let Some(id) = id_to_select {
            self.selected_id = Some(id);
        }

        let editor_x = content_x + sidebar_w + 15.0;
        let editor_w = box_w - sidebar_w - 30.0;
        let editor_y = box_y + 20.0;

        let sel_id = self.selected_id;
        let mut delete_target = None;
        let mut should_save = false;
        let mut old_title_to_remove = None;

        if let Some(sel) = sel_id {
            let current_note_title = self.notes.iter().find(|n| n.id == sel).map(|n| n.title.clone()).unwrap_or_default();

            let del_btn_rect = Rect::new(editor_x + editor_w - 80.0, editor_y + 18.0, 80.0, 28.0);
            if gui_button(del_btn_rect, "Delete", mouse_pos) {
                delete_target = Some(sel);
            }

            if let Some(note) = self.notes.iter_mut().find(|n| n.id == sel) {
                draw_text("Title:", editor_x, editor_y + 10.0, 12.0, LIGHTGRAY);

                let title_rect = Rect::new(editor_x, editor_y + 18.0, editor_w - 90.0, 28.0);
                let title_focused = self.focused_field == FocusedField::Title;

                let title_bg = if title_focused { Color::new(0.12, 0.18, 0.28, 1.0) } else { Color::new(0.08, 0.1, 0.16, 1.0) };
                let title_border = if title_focused { GREEN } else { SKYBLUE };

                draw_rectangle(title_rect.x, title_rect.y, title_rect.w, title_rect.h, title_bg);
                draw_rectangle_lines(title_rect.x, title_rect.y, title_rect.w, title_rect.h, 1.5, title_border);
                draw_text(&note.title, title_rect.x + 8.0, title_rect.y + 19.0, 13.0, WHITE);

                if is_mouse_button_pressed(MouseButton::Left) && title_rect.contains(mouse_pos.into()) {
                    self.focused_field = FocusedField::Title;
                }

                draw_text("Content:", editor_x, editor_y + 65.0, 12.0, LIGHTGRAY);
                let content_rect = Rect::new(editor_x, editor_y + 75.0, editor_w, box_h - 100.0);
                let content_focused = self.focused_field == FocusedField::Content;

                let content_bg = if content_focused { Color::new(0.1, 0.14, 0.2, 1.0) } else { Color::new(0.08, 0.1, 0.16, 1.0) };
                let content_border = if content_focused { GREEN } else { Color::new(0.2, 0.3, 0.4, 1.0) };

                draw_rectangle(content_rect.x, content_rect.y, content_rect.w, content_rect.h, content_bg);
                draw_rectangle_lines(content_rect.x, content_rect.y, content_rect.w, content_rect.h, 1.5, content_border);

                if is_mouse_button_pressed(MouseButton::Left) && content_rect.contains(mouse_pos.into()) {
                    self.focused_field = FocusedField::Content;
                }

                let chars = get_char_pressed();
                for c in chars {
                    if c == '\u{8}' {
                        if self.focused_field == FocusedField::Title {
                            if !note.title.is_empty() {
                                old_title_to_remove = Some(current_note_title.clone());
                                note.title.pop();
                            }
                        } else {
                            note.content.pop();
                        }
                        should_save = true;
                    } else if c == '\r' || c == '\n' {
                        if self.focused_field == FocusedField::Content {
                            note.content.push('\n');
                            should_save = true;
                        }
                    } else if !c.is_control() {
                        if self.focused_field == FocusedField::Title {
                            old_title_to_remove = Some(current_note_title.clone());
                            note.title.push(c);
                        } else {
                            note.content.push(c);
                        }
                        should_save = true;
                    }
                }

                let mut line_y = content_rect.y + 20.0;
                for line in note.content.lines() {
                    draw_text(line, content_rect.x + 10.0, line_y, 14.0, WHITE);
                    line_y += 18.0;
                    if line_y > content_rect.y + content_rect.h - 10.0 { break; }
                }

                if let Some(old_t) = old_title_to_remove {
                    let old_path = format!("{}/{}.txt", STORAGE_DIR, old_t);
                    if Path::new(&old_path).exists() {
                        let _ = fs::remove_file(old_path);
                    }
                }
            }
        } else {
            draw_text("No note selected. Click '+ New Note' to start.", editor_x + 20.0, editor_y + 100.0, 14.0, GRAY);
        }

        if let Some(del_id) = delete_target {
            if let Some(pos) = self.notes.iter().position(|n| n.id == del_id) {
                let note_to_delete = self.notes.remove(pos);
                let path = format!("{}/{}.txt", STORAGE_DIR, note_to_delete.title);
                if Path::new(&path).exists() {
                    let _ = fs::remove_file(path);
                }
            }
            self.selected_id = self.notes.first().map(|n| n.id);
            self.status_message = "Note deleted.".to_string();
        }

        if should_save {
            self.save_to_disk();
        }

        draw_text(&self.status_message, content_x, box_y + box_h + 20.0, 13.0, GREEN);
    }
}

fn gui_button(rect: Rect, text: &str, mouse: (f32, f32)) -> bool {
    let hovered = rect.contains(mouse.into());
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let col = if hovered { Color::new(0.3, 0.5, 0.4, 1.0) } else { Color::new(0.2, 0.3, 0.25, 1.0) };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, col);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, WHITE);

    let text_x = rect.x + (rect.w / 2.0) - (text.len() as f32 * 3.5);
    let text_y = rect.y + 18.0;
    draw_text(text, text_x, text_y, 13.0, WHITE);

    clicked
}
// note persistence sync
