use macroquad::prelude::*;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct NetworkState {
    pub peer_ip: String,
    pub is_connected: bool,
    pub connection_status: String,
    pub discovered_peers: Vec<String>,
    pub transfer_status: String,
    pub progress: f32,

    discovery_rx: Option<Receiver<String>>,
    transfer_rx: Option<Receiver<String>>,
    is_advertising: bool,
}

const DISCOVERY_PORT: u16 = 8888;
const TRANSFER_PORT: u16 = 8889;
const DOWNLOADS_DIR: &str = "downloads";

impl NetworkState {
    pub fn new() -> Self {
        let mut state = Self {
            peer_ip: "127.0.0.1".to_string(),
            is_connected: false,
            connection_status: "Offline - Ready to discover network.".to_string(),
            discovered_peers: Vec::new(),
            transfer_status: "Idle".to_string(),
            progress: 0.0,
            discovery_rx: None,
            transfer_rx: None,
            is_advertising: false,
        };

        state.start_tcp_listener();
        state.start_discovery_responder();
        state
    }

    fn start_discovery_responder(&mut self) {
        thread::spawn(move || {
            if let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)) {
                let _ = socket.set_broadcast(true);
                let mut buf = [0; 1024];
                loop {
                    if let Ok((_, src_addr)) = socket.recv_from(&mut buf) {
                        let _ = socket.send_to(b"NEXUS_DISCOVERY_PONG", src_addr);
                    }
                }
            }
        });
    }

    fn start_tcp_listener(&mut self) {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();
        self.transfer_rx = Some(rx);

        thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", TRANSFER_PORT)) {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let _ = tx.send("Incoming file connection detected...".to_string());

                        if !Path::new(DOWNLOADS_DIR).exists() {
                            let _ = fs::create_dir_all(DOWNLOADS_DIR);
                        }

                        let mut len_buf = [0u8; 4];
                        if stream.read_exact(&mut len_buf).is_err() {
                            let _ = tx.send("Error: Failed to read file name length.".to_string());
                            continue;
                        }
                        let name_len = u32::from_be_bytes(len_buf) as usize;

                        let mut name_buf = vec![0u8; name_len];
                        if stream.read_exact(&mut name_buf).is_err() {
                            let _ = tx.send("Error: Failed to read file name.".to_string());
                            continue;
                        }
                        let file_name = String::from_utf8_lossy(&name_buf).to_string();

                        let mut size_buf = [0u8; 8];
                        if stream.read_exact(&mut size_buf).is_err() {
                            let _ = tx.send("Error: Failed to read file size.".to_string());
                            continue;
                        }
                        let total_size = u64::from_be_bytes(size_buf);

                        let file_path = format!("{}/{}", DOWNLOADS_DIR, file_name);
                        if let Ok(mut file) = fs::File::create(&file_path) {
                            let mut buffer = [0; 8192];
                            let mut received_bytes: u64 = 0;

                            let mut success = true;
                            while received_bytes < total_size {
                                let to_read = std::cmp::min(buffer.len() as u64, total_size - received_bytes) as usize;
                                match stream.read(&mut buffer[..to_read]) {
                                    Ok(0) => {
                                        success = false;
                                        break;
                                    }
                                    Ok(n) => {
                                        if file.write_all(&buffer[..n]).is_err() {
                                            success = false;
                                            break;
                                        }
                                        received_bytes += n as u64;
                                    }
                                    Err(_) => {
                                        success = false;
                                        break;
                                    }
                                }
                            }

                            if success {
                                let _ = tx.send(format!("File '{}' received successfully! ({} bytes)", file_name, received_bytes));
                            } else {
                                let _ = tx.send("Error: File transfer interrupted.".to_string());
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn start_discovery(&mut self) {
        let (tx, rx) = channel();
        self.discovery_rx = Some(rx);
        self.discovered_peers.clear();
        self.connection_status = "Scanning local network via UDP...".to_string();

        let tx_clone = tx;

        thread::spawn(move || {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
                let _ = socket.set_broadcast(true);
                let _ = socket.set_read_timeout(Some(std::time::Duration::from_secs(2)));

                let _ = socket.send_to(b"NEXUS_DISCOVERY_PING", format!("255.255.255.255:{}", DISCOVERY_PORT));

                let mut buf = [0; 1024];
                let start_time = std::time::Instant::now();

                while start_time.elapsed().as_secs() < 2 {
                    if let Ok((_, src_addr)) = socket.recv_from(&mut buf) {
                        let ip_str = src_addr.ip().to_string();
                        let _ = tx_clone.send(ip_str);
                    }
                }
            }
        });
    }

    pub fn select_and_send_file(&mut self, target_ip: &str) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let file_path_str = path.to_string_lossy().to_string();
            self.send_file_tcp(&file_path_str, target_ip);
        } else {
            self.transfer_status = "File selection cancelled.".to_string();
        }
    }

    pub fn send_file_tcp(&mut self, file_path: &str, target_ip: &str) {
        let path = file_path.to_string();
        let ip = format!("{}:{}", target_ip, TRANSFER_PORT);
        let (tx, rx) = channel();
        self.transfer_rx = Some(rx);
        self.transfer_status = "Connecting to target peer...".to_string();

        thread::spawn(move || {
            let path_obj = Path::new(&path);
            let file_name = match path_obj.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => "unknown_file.bin".to_string(),
            };

            if let Ok(mut stream) = TcpStream::connect(&ip) {
                if let Ok(content) = fs::read(&path) {
                    let total_size = content.len() as u64;
                    let name_bytes = file_name.as_bytes();
                    let name_len = name_bytes.len() as u32;

                    let _ = tx.send(format!("Sending '{}' (Size: {} bytes)...", file_name, total_size));

                    if stream.write_all(&name_len.to_be_bytes()).is_err() {
                        let _ = tx.send("Error: Failed to send file metadata.".to_string());
                        return;
                    }
                    if stream.write_all(name_bytes).is_err() {
                        let _ = tx.send("Error: Failed to send file name.".to_string());
                        return;
                    }
                    if stream.write_all(&total_size.to_be_bytes()).is_err() {
                        let _ = tx.send("Error: Failed to send file size.".to_string());
                        return;
                    }

                    let mut sent = 0;
                    for chunk in content.chunks(8192) {
                        if stream.write_all(chunk).is_err() {
                            let _ = tx.send("Error: Failed to send data chunk.".to_string());
                            return;
                        }
                        sent += chunk.len();
                        let percent = (sent as f32 / total_size as f32) * 100.0;
                        let _ = tx.send(format!("PROGRESS:{}", percent));
                    }
                    let _ = tx.send("File transfer completed successfully!".to_string());
                } else {
                    let _ = tx.send("Error: File not found or unreadable.".to_string());
                }
            } else {
                let _ = tx.send("Error: Connection refused by target peer.".to_string());
            }
        });
    }

    pub fn update(&mut self) {
        if let Some(ref rx) = self.discovery_rx {
            while let Ok(ip) = rx.try_recv() {
                if ip != "127.0.0.1" && !ip.starts_with("0.0.0") && !self.discovered_peers.contains(&ip) {
                    self.discovered_peers.push(ip);
                }
            }

            if !self.discovered_peers.is_empty() {
                self.connection_status = format!("Found {} active peer(s) on network.", self.discovered_peers.len());
            } else {
                self.connection_status = "No peers found yet. Try scanning again.".to_string();
            }
        }

        if let Some(ref rx) = self.transfer_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.starts_with("PROGRESS:") {
                    if let Ok(p) = msg.strip_prefix("PROGRESS:").unwrap().parse::<f32>() {
                        self.progress = p;
                        self.transfer_status = format!("Transferring... {:.1}%", p);
                    }
                } else {
                    self.transfer_status = msg;
                }
            }
        }
    }
}