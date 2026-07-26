use macroquad::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::time::Duration;

#[derive(Clone)]
pub struct TrackInfo {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub cover_texture: Option<Texture2D>,
}

pub struct MusicLibraryState {
    pub current_track: usize,
    pub is_playing: bool,
    pub tracks: Vec<TrackInfo>,
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
    scan_status: String,
    target_dir: String,

    playback_duration: Duration,
    playback_position: Duration,
    track_started_instant: f64, 
}

impl MusicLibraryState {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().ok().unzip();
        let sink = stream_handle.as_ref().and_then(|h| Sink::try_new(h).ok());

        let default_dir = directories::UserDirs::new()
            .and_then(|u| u.download_dir().map(|p| p.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "C:/Users/Public/Downloads".to_string());

        let mut music_state = Self {
            current_track: 0,
            is_playing: false,
            tracks: Vec::new(),
            _stream,
            sink,
            scan_status: String::new(),
            target_dir: default_dir,
            playback_duration: Duration::from_secs(0),
            playback_position: Duration::from_secs(0),
            track_started_instant: 0.0,
        };

        let dir_to_scan = music_state.target_dir.clone();
        music_state.scan_music_directory(&dir_to_scan);

        music_state
    }

    pub fn scan_music_directory(&mut self, dir_path: &str) {
        self.tracks.clear();
        println!("[DEBUG] Smart scanning directory: {}", dir_path);

        let path_obj = Path::new(dir_path);
        if !path_obj.exists() {
            self.scan_status = format!("Directory not found: {}", dir_path);
            return;
        }

        match fs::read_dir(dir_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                let ext_str = ext.to_string_lossy().to_lowercase();
                                if ext_str == "mp3" || ext_str == "wav" || ext_str == "ogg" || ext_str == "flac" {
                                    if let Some(path_str) = path.to_str() {
                                        let mut title = path.file_name().unwrap().to_string_lossy().into_owned();
                                        let mut artist = "Unknown Artist".to_string();
                                        let mut album = "Unknown Album".to_string();
                                        let mut year = "----".to_string();
                                        let mut cover_texture = None;

                                        if let Ok(tagged_file) = Probe::open(&path).and_then(|p| p.read()) {
                                            if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.tags().first()) {
                                                if let Some(t) = tag.title() { title = t.to_string(); }
                                                if let Some(a) = tag.artist() { artist = a.to_string(); }
                                                if let Some(alb) = tag.album() { album = alb.to_string(); }
                                                if let Some(y) = tag.year() { year = y.to_string(); }

                                                if let Some(picture) = tag.pictures().first() {
                                                    let image_data = picture.data();
                                                    if let Ok(img) = image::load_from_memory(image_data) {
                                                        let rgba = img.to_rgba8();
                                                        let width = rgba.width() as u16;
                                                        let height = rgba.height() as u16;
                                                        cover_texture = Some(Texture2D::from_rgba8(width, height, &rgba));
                                                    }
                                                }
                                            }
                                        }

                                        self.tracks.push(TrackInfo {
                                            file_path: path_str.to_string(),
                                            title,
                                            artist,
                                            album,
                                            year,
                                            cover_texture,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                if self.tracks.is_empty() {
                    self.scan_status = format!("No audio found in: {}", dir_path);
                } else {
                    self.scan_status = format!("Loaded {} track(s) successfully!", self.tracks.len());
                }
            }
            Err(e) => {
                self.scan_status = format!("Error reading folder: {}", e);
            }
        }
    }

    fn play_current_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if let Some(ref sink) = self.sink {
            sink.stop();

            let track_path = &self.tracks[self.current_track].file_path;
            if let Ok(file) = File::open(track_path) {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    self.playback_duration = source.total_duration().unwrap_or(Duration::from_secs(180));
                    self.playback_position = Duration::from_secs(0);
                    self.track_started_instant = get_time();

                    sink.append(source);
                    sink.play();
                    self.is_playing = true;
                } else {
                    println!("[ERROR] Could not decode audio: {}", track_path);
                }
            } else {
                println!("[ERROR] Could not open file: {}", track_path);
            }
        }
    }

    fn seek_to_position(&mut self, target_progress: f32) {
        if self.tracks.is_empty() {
            return;
        }

        let total_secs = self.playback_duration.as_secs_f32();
        let target_secs = total_secs * target_progress;
        self.playback_position = Duration::from_secs_f32(target_secs);
        self.track_started_instant = get_time() - target_secs as f64;

        if let Some(ref sink) = self.sink {
            sink.stop();

            let track_path = &self.tracks[self.current_track].file_path;
            if let Ok(file) = File::open(track_path) {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    let target_duration = Duration::from_secs_f32(target_secs);
                    let skipped_source = source.skip_duration(target_duration);

                    sink.append(skipped_source);
                    if self.is_playing {
                        sink.play();
                    }
                }
            }
        }
    }

    pub fn toggle_play_pause(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if let Some(ref sink) = self.sink {
            if self.is_playing {
                sink.pause();
                self.is_playing = false;
            } else {
                if sink.empty() {
                    self.play_current_track();
                } else {
                    sink.play();
                    self.is_playing = true;
                }
            }
        }
    }

    pub fn update(&mut self, content_x: f32) {
        draw_text("🎵 Smart Music Center (Real Seek & Duration)", content_x, 95.0, 20.0, MAGENTA);
        draw_text(&self.scan_status, content_x, 120.0, 13.0, SKYBLUE);

        if self.is_playing {
            let elapsed = get_time() - self.track_started_instant;
            self.playback_position = Duration::from_secs_f32(elapsed as f32);

            if self.playback_position >= self.playback_duration {
                self.playback_position = self.playback_duration;
                self.is_playing = false;
            }
        }

        let mut ty = 155.0;
        let mouse_pos = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        let mut track_to_play: Option<usize> = None;

        if self.tracks.is_empty() {
            draw_text("No tracks available. Press [R] to scan.", content_x, ty + 20.0, 13.0, YELLOW);
        } else {
            let tracks_len = self.tracks.len();
            for idx in 0..tracks_len {
                let track = &self.tracks[idx];
                let track_rect = Rect::new(content_x, ty, 620.0, 48.0);
                let is_hovered = track_rect.contains(mouse_pos.into());
                let is_selected = self.current_track == idx;

                let bg_col = if is_selected {
                    Color::new(0.25, 0.15, 0.4, 1.0)
                } else if is_hovered {
                    Color::new(0.18, 0.12, 0.3, 1.0)
                } else {
                    Color::new(0.12, 0.08, 0.22, 1.0)
                };

                draw_rectangle(track_rect.x, track_rect.y, track_rect.w, track_rect.h, bg_col);
                draw_rectangle_lines(track_rect.x, track_rect.y, track_rect.w, track_rect.h, 1.0, Color::new(0.3, 0.2, 0.5, 1.0));

                let display_title = if is_selected && self.is_playing {
                    format!("{} - {} [ ▶ Playing ]", track.artist, track.title)
                } else {
                    format!("{} - {}", track.artist, track.title)
                };

                let meta_info = format!("Album: {} | Year: {}", track.album, track.year);

                draw_text(&display_title, content_x + 15.0, ty + 20.0, 14.0, WHITE);
                draw_text(&meta_info, content_x + 15.0, ty + 38.0, 11.0, LIGHTGRAY);

                if is_hovered && clicked {
                    track_to_play = Some(idx);
                }

                ty += 56.0;
            }
        }

        if let Some(idx) = track_to_play {
            self.current_track = idx;
            self.play_current_track();
        }

        if is_key_pressed(KeyCode::R) {
            let dir = self.target_dir.clone();
            self.scan_music_directory(&dir);
        }

        let control_y = screen_height() - 140.0;
        draw_rectangle(content_x, control_y, 620.0, 115.0, Color::new(0.08, 0.05, 0.15, 1.0));
        draw_rectangle_lines(content_x, control_y, 620.0, 115.0, 1.0, Color::new(0.3, 0.15, 0.45, 1.0));

        if !self.tracks.is_empty() {
            let current_artist = self.tracks[self.current_track].artist.clone();
            let current_title = self.tracks[self.current_track].title.clone();
            let cover_texture = self.tracks[self.current_track].cover_texture.clone();

            let cover_rect = Rect::new(content_x + 12.0, control_y + 12.0, 90.0, 90.0);
            draw_rectangle(cover_rect.x, cover_rect.y, cover_rect.w, cover_rect.h, Color::new(0.04, 0.02, 0.08, 1.0));

            if let Some(ref texture) = cover_texture {
                draw_texture_ex(
                    texture,
                    cover_rect.x,
                    cover_rect.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(90.0, 90.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_text("No Art", cover_rect.x + 28.0, cover_rect.y + 50.0, 12.0, GRAY);
            }
            draw_rectangle_lines(cover_rect.x, cover_rect.y, cover_rect.w, cover_rect.h, 1.0, Color::new(0.3, 0.2, 0.4, 1.0));

            let play_pause_btn = Rect::new(content_x + 115.0, control_y + 15.0, 80.0, 30.0);
            let btn_hovered = play_pause_btn.contains(mouse_pos.into());
            let btn_col = if btn_hovered { Color::new(0.4, 0.2, 0.6, 1.0) } else { Color::new(0.3, 0.15, 0.45, 1.0) };

            draw_rectangle(play_pause_btn.x, play_pause_btn.y, play_pause_btn.w, play_pause_btn.h, btn_col);
            let btn_label = if self.is_playing { "Pause" } else { "Play" };
            draw_text(btn_label, play_pause_btn.x + 24.0, play_pause_btn.y + 20.0, 13.0, WHITE);

            if btn_hovered && clicked {
                self.toggle_play_pause();
            }

            let info_msg = format!("Playing: {} - {}", current_artist, current_title);
            draw_text(&info_msg, content_x + 210.0, control_y + 35.0, 13.0, SKYBLUE);

            let current_secs = self.playback_position.as_secs_f32();
            let total_secs = self.playback_duration.as_secs_f32();
            let progress_ratio = if total_secs > 0.0 { (current_secs / total_secs).clamp(0.0, 1.0) } else { 0.0 };

            let bar_x = content_x + 115.0;
            let bar_y = control_y + 75.0;
            let bar_w = 485.0;
            let bar_h = 10.0;

            let seek_rect = Rect::new(bar_x, bar_y - 5.0, bar_w, 20.0);
            let seek_hovered = seek_rect.contains(mouse_pos.into());

            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.15, 0.1, 0.25, 1.0));

            let filled_w = bar_w * progress_ratio;
            draw_rectangle(bar_x, bar_y, filled_w, bar_h, MAGENTA);
            draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, Color::new(0.4, 0.2, 0.6, 1.0));

            let cur_min = (current_secs as u32) / 60;
            let cur_sec = (current_secs as u32) % 60;
            let tot_min = (total_secs as u32) / 60;
            let tot_sec = (total_secs as u32) % 60;
            let time_str = format!("{:02}:{:02} / {:02}:{:02}", cur_min, cur_sec, tot_min, tot_sec);
            draw_text(&time_str, bar_x + bar_w - 90.0, bar_y - 8.0, 11.0, LIGHTGRAY);

            if seek_hovered && clicked {
                let click_x = mouse_pos.0 - bar_x;
                let clicked_ratio = (click_x / bar_w).clamp(0.0, 1.0);
                self.seek_to_position(clicked_ratio);
            }
        }

        draw_text("Press [R] to Rescan | [ESC] Dashboard", content_x, screen_height() - 15.0, 12.0, GRAY);
    }
}
// music playback sync
