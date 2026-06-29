//! # junqi-net — 军棋网络层
//!
//! 提供 P2P 网络对战能力，包括：
//! - 协议定义（JSON over TCP）
//! - Rendezvous 客户端（房间注册/查询）
//! - 房间管理（创建/加入/踢人/开始）
//! - 对局同步（走法传输）
pub mod protocol;
pub mod client;
pub mod room;
pub mod sync;
