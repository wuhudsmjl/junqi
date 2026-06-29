use junqi_core::moves::{Move, MoveResult};
use junqi_core::types::Color;
use serde::{Deserialize, Serialize};

/// 客户端与服务端之间的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 创建房间请求
    CreateRoom { player_name: String },
    /// 创建房间响应
    RoomCreated { room_code: u16 },
    /// 加入房间请求
    JoinRoom { room_code: u16, player_name: String },
    /// 加入房间成功
    RoomJoined { host_name: String },
    /// 新玩家加入通知（发给房主）
    PlayerJoined { player_name: String },
    /// 玩家离开
    PlayerLeft { player_name: String },
    /// 被踢出房间
    Kicked { reason: String },
    /// 房间已解散
    RoomDisbanded,
    /// 开始游戏（房主发起）
    StartGame,
    /// 游戏开始通知
    GameStarted { opponent_name: String, your_color: Color },
    /// 走法
    GameMove { mv: Move, step_number: u32 },
    /// 走法结果
    MoveResultMsg { mv: Move, result: MoveResult, step_number: u32 },
    /// 认输
    Surrender,
    /// 请求和棋
    RequestDraw,
    /// 和棋响应
    DrawResponse { accept: bool },
    /// 聊天消息
    ChatMessage { from: String, text: String },
    /// 错误消息
    Error { message: String },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

impl ServerMessage {
    /// 序列化为 JSON 行
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// 从 JSON 行反序列化
    pub fn from_line(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }
}
