use std::net::SocketAddr;
use std::time::Duration;

use crate::client::GameClient;
use crate::protocol::ServerMessage;
use crate::sync::SyncGameState;

/// 房间状态
#[derive(Debug, Clone, PartialEq)]
pub enum RoomState {
    /// 等待玩家加入
    Waiting,
    /// 对局进行中
    Playing,
    /// 已解散
    Disbanded,
}

/// 在线房间管理器（客户端侧）
pub struct OnlineRoom {
    /// 网络客户端
    pub client: GameClient,
    /// 房间号
    pub room_code: u16,
    /// 是否是房主
    pub is_host: bool,
    /// 房主名称
    pub host_name: String,
    /// 对方名称
    pub opponent_name: Option<String>,
    /// 房间状态
    pub state: RoomState,
    /// 我方在游戏中的颜色
    pub my_color: Option<junqi_core::types::Color>,
    /// 游戏同步状态
    pub game_sync: Option<SyncGameState>,
}

impl OnlineRoom {
    /// 创建房间（作为房主）
    pub fn create(player_name: &str, server_addr: SocketAddr) -> Result<Self, String> {
        let client = GameClient::connect(server_addr)?;

        client.send(ServerMessage::CreateRoom {
            player_name: player_name.to_string(),
        })?;

        match client.recv_timeout(Duration::from_secs(5)) {
            Some(ServerMessage::RoomCreated { room_code }) => {
                Ok(OnlineRoom {
                    client,
                    room_code,
                    is_host: true,
                    host_name: player_name.to_string(),
                    opponent_name: None,
                    state: RoomState::Waiting,
                    my_color: None,
                    game_sync: None,
                })
            }
            Some(ServerMessage::Error { message }) => Err(message),
            _ => Err("创建房间超时".to_string()),
        }
    }

    /// 加入房间（作为访客）
    pub fn join(room_code: u16, player_name: &str, server_addr: SocketAddr) -> Result<Self, String> {
        let client = GameClient::connect(server_addr)?;
        client.send(ServerMessage::JoinRoom {
            room_code,
            player_name: player_name.to_string(),
        })?;

        match client.recv_timeout(Duration::from_secs(5)) {
            Some(ServerMessage::RoomJoined { host_name }) => {
                let host = host_name.clone();
                Ok(OnlineRoom {
                    client,
                    room_code,
                    is_host: false,
                    host_name,
                    opponent_name: Some(host),
                    state: RoomState::Waiting,
                    my_color: None,
                    game_sync: None,
                })
            }
            Some(ServerMessage::Error { message }) => Err(message),
            _ => Err("加入房间超时或房间不存在".to_string()),
        }
    }

    /// 检查新消息（在 GUI 循环中调用）
    pub fn poll(&mut self) -> Vec<ServerMessage> {
        let mut msgs = Vec::new();
        while let Some(msg) = self.client.try_recv() {
            match &msg {
                ServerMessage::PlayerJoined { player_name } => {
                    self.opponent_name = Some(player_name.clone());
                }
                ServerMessage::GameStarted { opponent_name, your_color } => {
                    self.opponent_name = Some(opponent_name.clone());
                    self.my_color = Some(*your_color);
                    self.state = RoomState::Playing;
                }
                ServerMessage::PlayerLeft { .. } | ServerMessage::RoomDisbanded => {
                    self.state = RoomState::Disbanded;
                }
                ServerMessage::Kicked { .. } => {
                    self.state = RoomState::Disbanded;
                }
                _ => {}
            }
            msgs.push(msg);
        }
        msgs
    }

    /// 开始游戏（仅房主）
    pub fn start_game(&self) -> Result<(), String> {
        if !self.is_host {
            return Err("只有房主可以开始游戏".to_string());
        }
        self.client.send(ServerMessage::StartGame)
    }

    /// 发送走法
    pub fn send_move(&self, mv: &junqi_core::moves::Move, step_number: u32) -> Result<(), String> {
        self.client.send(ServerMessage::GameMove {
            mv: *mv,
            step_number,
        })
    }

    /// 发送认输
    pub fn surrender(&self) -> Result<(), String> {
        self.client.send(ServerMessage::Surrender)
    }

    /// 发送聊天消息
    pub fn send_chat(&self, text: &str) -> Result<(), String> {
        self.client.send(ServerMessage::ChatMessage {
            from: "我".to_string(),
            text: text.to_string(),
        })
    }

    /// 离开房间
    pub fn leave(&self) {}
}
