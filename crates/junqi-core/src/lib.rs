//! # junqi-core — 军棋陆战棋核心引擎
//!
//! 实现 CNVCS（中国网络象棋标准）军棋暗棋规则。
//!
//! ## 棋盘
//!
//! 5 列 × 12 行，每方 6 行 × 5 列。下标 (row, col) 从 0 开始。
//! - 红方领土: row 0–5
//! - 蓝方领土: row 6–11
//! - 前线: row 5 ↔ row 6
//!
//! ## 棋子（每方 25 枚）
//!
//! | 棋子 | 数量 | 等级 | 特性 |
//! |------|------|------|------|
//! | 司令 | 1 | 9 | — |
//! | 军长 | 1 | 8 | — |
//! | 师长 | 2 | 7 | — |
//! | 旅长 | 2 | 6 | — |
//! | 团长 | 2 | 5 | — |
//! | 营长 | 2 | 4 | — |
//! | 连长 | 3 | 3 | — |
//! | 排长 | 3 | 2 | — |
//! | 工兵 | 3 | 1 | 可沿铁路滑行、可挖地雷 |
//! | 炸弹 | 2 | 0 | 与对方同归于尽 |
//! | 地雷 | 3 | 0 | 不可移动、非工兵攻击即死 |
//! | 军旗 | 1 | 0 | 不可移动、被吃即输 |
//!
//! ## 暗棋规则
//!
//! - 初始时对方棋子不可见
//! - 碰撞时双方棋子互相揭示
//! - 司令阵亡时暴露己方军旗位置
//! - 行营内棋子不可被攻击
//! - 大本营只能放军旗，己方棋子不可进入
//! - 进入对方大本营后不可继续移动

pub mod board;
pub mod game;
pub mod layout;
pub mod moves;
pub mod piece;
pub mod replay;
pub mod types;

pub use board::Board;
pub use game::{Game, GameError, GamePhase, GameResult, WinReason};
pub use layout::Layout;
pub use moves::{Move, MoveResult};
pub use piece::{CombatResult, Piece};
pub use replay::{Replay, Step};
pub use types::{CellType, Color, Direction, PieceKind, Position, AiDifficulty};
