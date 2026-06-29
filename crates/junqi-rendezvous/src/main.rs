use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use junqi_net::protocol::ServerMessage;

/// 房间信息
struct Room {
    /// 房间号
    _code: u16,
    /// 房主名称
    host_name: String,
    /// 房主连接（用索引表示，因 TcpStream 不实现 Clone）
    host_idx: usize,
    /// 访客连接索引（None 表示等待中）
    guest_idx: Option<usize>,
    /// 创建时间
    created_at: Instant,
}

/// 玩家连接
struct Player {
    stream: TcpStream,
    name: String,
    room_code: Option<u16>,
}

/// 服务器状态
struct ServerState {
    rooms: HashMap<u16, Room>,
    players: Vec<Player>,
    next_room_code: u16,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            rooms: HashMap::new(),
            players: Vec::new(),
            next_room_code: 1000,
        }
    }

    /// 生成唯一房间号
    fn generate_room_code(&mut self) -> u16 {
        loop {
            let code = self.next_room_code;
            self.next_room_code = if self.next_room_code >= 9999 { 1000 } else { self.next_room_code + 1 };
            if !self.rooms.contains_key(&code) {
                return code;
            }
        }
    }

    /// 处理消息
    fn handle_message(&mut self, player_idx: usize, msg: ServerMessage) -> Option<ServerMessage> {
        match msg {
            ServerMessage::CreateRoom { player_name } => {
                let code = self.generate_room_code();
                if let Some(player) = self.players.get_mut(player_idx) {
                    player.name = player_name.clone();
                    player.room_code = Some(code);
                }
                self.rooms.insert(code, Room {
                    _code: code,
                    host_name: player_name.clone(),
                    host_idx: player_idx,
                    guest_idx: None,
                    created_at: Instant::now(),
                });
                log::info!("房间 {} 创建，房主: {}", code, player_name);
                Some(ServerMessage::RoomCreated { room_code: code })
            }

            ServerMessage::JoinRoom { room_code, player_name } => {
                if let Some(room) = self.rooms.get_mut(&room_code) {
                    if room.guest_idx.is_some() {
                        return Some(ServerMessage::Error { message: "房间已满".to_string() });
                    }
                    room.guest_idx = Some(player_idx);
                    if let Some(player) = self.players.get_mut(player_idx) {
                        player.name = player_name.clone();
                        player.room_code = Some(room_code);
                    }
                    if let Some(host) = self.players.get(room.host_idx) {
                        let msg = ServerMessage::PlayerJoined { player_name: player_name.clone() };
                        Self::send_to_player(host, &msg);
                    }
                    log::info!("玩家 {} 加入房间 {}", player_name, room_code);
                    Some(ServerMessage::RoomJoined { host_name: room.host_name.clone() })
                } else {
                    Some(ServerMessage::Error { message: "房间不存在".to_string() })
                }
            }

            ServerMessage::StartGame => {
                if let Some(player) = self.players.get(player_idx) {
                    if let Some(room_code) = player.room_code {
                        if let Some(room) = self.rooms.get(&room_code) {
                            if player_idx != room.host_idx {
                                return Some(ServerMessage::Error { message: "只有房主可以开始游戏".to_string() });
                            }
                            if let Some(host) = self.players.get(room.host_idx) {
                                Self::send_to_player(host, &ServerMessage::GameStarted {
                                    opponent_name: room.guest_idx
                                        .and_then(|i| self.players.get(i))
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default(),
                                    your_color: junqi_core::types::Color::Red,
                                });
                            }
                            if let Some(guest_idx) = room.guest_idx {
                                if let Some(guest) = self.players.get(guest_idx) {
                                    Self::send_to_player(guest, &ServerMessage::GameStarted {
                                        opponent_name: room.host_name.clone(),
                                        your_color: junqi_core::types::Color::Blue,
                                    });
                                }
                            }
                            log::info!("房间 {} 游戏开始", room_code);
                        }
                    }
                }
                None
            }

            ServerMessage::GameMove { .. }
            | ServerMessage::Surrender
            | ServerMessage::RequestDraw
            | ServerMessage::DrawResponse { .. }
            | ServerMessage::ChatMessage { .. } => {
                self.forward_to_opponent(player_idx, &msg);
                None
            }

            ServerMessage::Ping => Some(ServerMessage::Pong),
            _ => None,
        }
    }

    /// 转发消息给对手
    fn forward_to_opponent(&self, from_idx: usize, msg: &ServerMessage) {
        if let Some(player) = self.players.get(from_idx) {
            if let Some(room_code) = player.room_code {
                if let Some(room) = self.rooms.get(&room_code) {
                    let target_idx = if from_idx == room.host_idx {
                        room.guest_idx
                    } else {
                        Some(room.host_idx)
                    };
                    if let Some(target_idx) = target_idx {
                        if let Some(target) = self.players.get(target_idx) {
                            Self::send_to_player(target, msg);
                        }
                    }
                }
            }
        }
    }

    /// 发送消息给指定玩家
    fn send_to_player(player: &Player, msg: &ServerMessage) {
        let mut stream = &player.stream;
        let mut line = serde_json::to_string(msg).unwrap_or_default();
        line.push('\n');
        let _ = stream.write_all(line.as_bytes());
        let _ = stream.flush();
    }

    /// 处理玩家断开连接
    fn handle_disconnect(&mut self, player_idx: usize) {
        if let Some(player) = self.players.get(player_idx) {
            if let Some(room_code) = player.room_code {
                self.forward_to_opponent(player_idx, &ServerMessage::PlayerLeft {
                    player_name: player.name.clone(),
                });
                if let Some(room) = self.rooms.get(&room_code) {
                    if room.guest_idx == Some(player_idx) {
                        if let Some(room) = self.rooms.get_mut(&room_code) {
                            room.guest_idx = None;
                        }
                    } else if room.host_idx == player_idx {
                        self.rooms.remove(&room_code);
                        log::info!("房间 {} 已解散", room_code);
                    }
                }
            }
        }
    }

    /// 清理超时房间
    fn cleanup(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(600);
        self.rooms.retain(|code, room| {
            let expired = now.duration_since(room.created_at) > timeout && room.guest_idx.is_none();
            if expired {
                log::info!("清理超时房间 {}", code);
            }
            !expired
        });
    }
}

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:9876").expect("绑定端口 9876 失败");
    println!("军棋服务器已启动，监听端口 9876");

    let state = Arc::new(Mutex::new(ServerState::new()));

    {
        let state = state.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(60));
            if let Ok(mut s) = state.lock() {
                s.cleanup();
            }
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                thread::spawn(move || {
                    handle_client(state, stream);
                });
            }
            Err(e) => {
                log::error!("接受连接失败: {}", e);
            }
        }
    }
}

fn handle_client(state: Arc<Mutex<ServerState>>, stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
    log::info!("新连接: {}", peer_addr);

    let player_idx;
    {
        let mut s = state.lock().unwrap();
        player_idx = s.players.len();
        s.players.push(Player {
            stream: stream.try_clone().expect("clone"),
            name: String::new(),
            room_code: None,
        });
    }

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Some(msg) = ServerMessage::from_line(trimmed) {
                    let response = {
                        let mut s = state.lock().unwrap();
                        s.handle_message(player_idx, msg)
                    };
                    if let Some(resp) = response {
                        let s = state.lock().unwrap();
                        if let Some(player) = s.players.get(player_idx) {
                            ServerState::send_to_player(player, &resp);
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    {
        let mut s = state.lock().unwrap();
        s.handle_disconnect(player_idx);
    }
    log::info!("断开连接: {}", peer_addr);
}
