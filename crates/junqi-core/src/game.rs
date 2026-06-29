use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::layout::Layout;
use crate::moves::{execute_move, generate_moves, is_legal_move, Move, MoveResult};
use crate::replay::{Replay, Step};
use crate::types::{Color, PieceKind, Position};

/// 对局阶段
///
/// 对局经历的阶段状态机：
/// `NotStarted` -> `Deploying` -> `Playing` -> `Finished`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    /// 对局尚未开始，等待布阵
    NotStarted,
    /// 布阵阶段（一方或双方正在布置棋子）
    Deploying,
    /// 对局进行中，双方轮流行棋
    Playing,
    /// 对局已结束
    Finished,
}

/// 对局结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    /// 红方获胜
    RedWins { reason: WinReason },
    /// 蓝方获胜
    BlueWins { reason: WinReason },
    /// 平局
    Draw { reason: DrawReason },
}

/// 获胜原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinReason {
    /// 夺得对方军旗
    CapturedFlag,
    /// 对方无棋可走
    NoMovesLeft,
    /// 对方认输
    Surrender,
    /// 对方超时
    Timeout,
}

/// 平局原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawReason {
    /// 双方同意和棋
    Agreed,
    /// 双方均无进攻能力
    NoAttackPieces,
}

/// 对局管理器，负责维护对局的完整状态。
///
/// 包含棋盘、当前阶段、当前走子方、历史记录、布阵方案和结果。
/// 提供布阵部署、走法执行、认输、复盘导出/导入等功能。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// 棋盘状态
    pub board: Board,
    /// 当前对局阶段
    pub phase: GamePhase,
    /// 当前走子方颜色
    pub current_turn: Color,
    /// 历史走法记录列表
    pub history: Vec<Step>,
    /// 走法步数计数器
    pub step_count: u32,
    /// 红方布阵方案
    pub red_layout: Option<Layout>,
    /// 蓝方布阵方案
    pub blue_layout: Option<Layout>,
    /// 对局结果（仅在 `Finished` 阶段可用）
    pub result: Option<GameResult>,
}

impl Game {
    /// 创建一个新的对局，初始状态为 `NotStarted`。
    ///
    /// 棋盘为空，当前走子方为红方。
    ///
    /// # 返回值
    /// 一个新的 `Game` 实例
    pub fn new() -> Self {
        Game {
            board: Board::new(),
            phase: GamePhase::NotStarted,
            current_turn: Color::Red,
            history: Vec::new(),
            step_count: 0,
            red_layout: None,
            blue_layout: None,
            result: None,
        }
    }

    /// 设置红方布阵方案。
    ///
    /// 验证布阵合法性后应用到棋盘上，将对局推进到 `Deploying` 阶段，
    /// 并将当前走子方切换为蓝方。
    ///
    /// # 参数
    /// - `layout`: 红方布阵方案
    ///
    /// # 返回值
    /// 成功返回 `Ok(())`，布阵非法返回对应的 `LayoutError`
    pub fn deploy_red(&mut self, layout: &Layout) -> Result<(), crate::layout::LayoutError> {
        layout.validate(Color::Red)?;
        layout.apply_to_board(&mut self.board, Color::Red);
        self.red_layout = Some(layout.clone());
        self.phase = GamePhase::Deploying;
        self.current_turn = Color::Blue;
        Ok(())
    }

    /// 设置蓝方布阵方案并开始对局。
    ///
    /// 验证布阵合法性后应用到棋盘上，将对局推进到 `Playing` 阶段，
    /// 并将当前走子方设为红方（红方先手）。
    ///
    /// # 参数
    /// - `layout`: 蓝方布阵方案
    ///
    /// # 返回值
    /// 成功返回 `Ok(())`，布阵非法返回对应的 `LayoutError`
    pub fn deploy_blue(&mut self, layout: &Layout) -> Result<(), crate::layout::LayoutError> {
        layout.validate(Color::Blue)?;
        layout.apply_to_board(&mut self.board, Color::Blue);
        self.blue_layout = Some(layout.clone());
        self.phase = GamePhase::Playing;
        self.current_turn = Color::Red;
        Ok(())
    }

    /// 同时设置双方布阵方案并直接开始对局。
    ///
    /// 跳过 `Deploying` 阶段，双方布阵后直接进入 `Playing` 阶段，
    /// 当前走子方为红方。
    ///
    /// # 参数
    /// - `red_layout`: 红方布阵方案
    /// - `blue_layout`: 蓝方布阵方案
    ///
    /// # 返回值
    /// 成功返回 `Ok(())`，任一布阵非法返回对应的 `LayoutError`
    pub fn deploy_both(
        &mut self,
        red_layout: &Layout,
        blue_layout: &Layout,
    ) -> Result<(), crate::layout::LayoutError> {
        red_layout.validate(Color::Red)?;
        blue_layout.validate(Color::Blue)?;
        red_layout.apply_to_board(&mut self.board, Color::Red);
        blue_layout.apply_to_board(&mut self.board, Color::Blue);
        self.red_layout = Some(red_layout.clone());
        self.blue_layout = Some(blue_layout.clone());
        self.phase = GamePhase::Playing;
        self.current_turn = Color::Red;
        Ok(())
    }

    /// 执行一步走法。
    ///
    /// 操作流程：
    /// 1. 检查当前阶段是否为 `Playing`
    /// 2. 检查走法是否合法
    /// 3. 执行走法并记录结果
    /// 4. 检查是否夺得军旗（获胜）
    /// 5. 切换走子方
    /// 6. 检查对方是否无棋可走（获胜）
    ///
    /// # 参数
    /// - `mv`: 要执行的走法
    ///
    /// # 返回值
    /// 成功返回 `Ok(MoveResult)`，失败返回对应的 `GameError`
    ///
    /// # 错误
    /// - `WrongPhase`: 当前不是 `Playing` 阶段
    /// - `IllegalMove`: 走法不合法
    /// - `GameOver`: 走法导致对局结束
    pub fn make_move(&mut self, mv: &Move) -> Result<MoveResult, GameError> {
        if self.phase != GamePhase::Playing {
            return Err(GameError::WrongPhase);
        }
        if !is_legal_move(&self.board, mv, self.current_turn) {
            return Err(GameError::IllegalMove);
        }

        let result = execute_move(&mut self.board, mv);

        let step = Step {
            step_number: self.step_count,
            mv: *mv,
            result: result.clone(),
            turn: self.current_turn,
        };
        self.history.push(step);
        self.step_count += 1;

        if result.captured_flag() {
            self.phase = GamePhase::Finished;
            let game_result = match self.current_turn {
                Color::Red => GameResult::RedWins { reason: WinReason::CapturedFlag },
                Color::Blue => GameResult::BlueWins { reason: WinReason::CapturedFlag },
            };
            self.result = Some(game_result.clone());
            return Err(GameError::GameOver(game_result));
        }

        self.current_turn = self.current_turn.opponent();

        if generate_moves(&self.board, self.current_turn).is_empty() {
            self.phase = GamePhase::Finished;
            let game_result = match self.current_turn.opponent() {
                Color::Red => GameResult::RedWins { reason: WinReason::NoMovesLeft },
                Color::Blue => GameResult::BlueWins { reason: WinReason::NoMovesLeft },
            };
            self.result = Some(game_result.clone());
            return Err(GameError::GameOver(game_result));
        }

        Ok(result)
    }

    /// 当前走子方认输，对局结束。
    ///
    /// 将阶段设为 `Finished`，对局结果为对方获胜（原因：认输）。
    ///
    /// # 返回值
    /// 对局结果（对方获胜）
    pub fn surrender(&mut self) -> GameResult {
        self.phase = GamePhase::Finished;
        let result = match self.current_turn {
            Color::Red => GameResult::BlueWins { reason: WinReason::Surrender },
            Color::Blue => GameResult::RedWins { reason: WinReason::Surrender },
        };
        self.result = Some(result.clone());
        result
    }

    /// 获取指定视角的可见棋盘视图。
    ///
    /// 暗棋规则下，对方未揭示的棋子种类不可见（返回 `None`）。
    ///
    /// # 参数
    /// - `viewer`: 观察者的颜色
    ///
    /// # 返回值
    /// 可见的棋盘视图 `VisibleBoard`
    pub fn visible_board(&self, viewer: Color) -> VisibleBoard {
        let mut pieces: Vec<(Position, Option<PieceKind>, Color)> = Vec::new();
        for row in 0..crate::board::ROWS {
            for col in 0..crate::board::COLS {
                let pos = Position::new(row as u8, col as u8);
                if let Some(piece) = self.board.piece_at(pos) {
                    let visible_kind = piece.visible_kind(viewer);
                    pieces.push((pos, visible_kind, piece.color));
                }
            }
        }
        VisibleBoard { pieces }
    }

    /// 获取当前走子方所有合法走法。
    ///
    /// # 返回值
    /// 合法走法列表，若非 `Playing` 阶段则返回空列表
    pub fn legal_moves(&self) -> Vec<Move> {
        if self.phase != GamePhase::Playing {
            return vec![];
        }
        generate_moves(&self.board, self.current_turn)
    }

    /// 将对局导出为复盘数据。
    ///
    /// # 参数
    /// - `red_name`: 红方玩家名称
    /// - `blue_name`: 蓝方玩家名称
    ///
    /// # 返回值
    /// 包含完整对局信息的 `Replay` 实例
    pub fn to_replay(&self, red_name: &str, blue_name: &str) -> Replay {
        Replay {
            red_name: red_name.to_string(),
            blue_name: blue_name.to_string(),
            red_layout: self.red_layout.clone(),
            blue_layout: self.blue_layout.clone(),
            steps: self.history.clone(),
            result: self.result.clone(),
            saved: false,
        }
    }

    /// 从复盘数据重建对局。
    ///
    /// 可选择只回放到指定的步数。
    ///
    /// # 参数
    /// - `replay`: 复盘数据
    /// - `up_to_step`: 回放到的步数（`None` 表示回放全部）
    ///
    /// # 返回值
    /// 成功返回重建的 `Game`，失败返回对应的 `GameError`
    ///
    /// # 错误
    /// - `NoLayout`: 复盘数据缺少布阵方案
    /// - `InvalidLayout`: 布阵方案不合法
    pub fn from_replay(replay: &Replay, up_to_step: Option<usize>) -> Result<Self, GameError> {
        let mut game = Game::new();
        let red_layout = replay.red_layout.as_ref().ok_or(GameError::NoLayout)?;
        let blue_layout = replay.blue_layout.as_ref().ok_or(GameError::NoLayout)?;
        game.deploy_both(red_layout, blue_layout)
            .map_err(|_| GameError::InvalidLayout)?;

        let max_step = up_to_step.unwrap_or(replay.steps.len());
        for step in replay.steps.iter().take(max_step) {
            let _ = game.make_move(&step.mv);
        }
        Ok(game)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// 指定视角下的可见棋盘视图
///
/// 包含棋盘上所有棋子及各自主人的颜色，
/// 但对方未揭示的棋子种类不可见（`Option<PieceKind>` 为 `None`）。
#[derive(Debug, Clone)]
pub struct VisibleBoard {
    /// 棋子列表，每项为 (位置, 可见种类, 所属颜色)
    pub pieces: Vec<(Position, Option<PieceKind>, Color)>,
}

/// 对局操作错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum GameError {
    /// 当前阶段不能执行此操作
    #[error("当前阶段不能执行此操作")]
    WrongPhase,
    /// 走法不合法
    #[error("非法走法")]
    IllegalMove,
    /// 游戏已结束
    #[error("游戏已结束：{0:?}")]
    GameOver(GameResult),
    /// 复盘数据缺少布阵方案
    #[error("缺少布阵数据")]
    NoLayout,
    /// 布阵方案不合法
    #[error("布阵无效")]
    InvalidLayout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{builtin_aggressive, builtin_defensive};

    #[test]
    fn test_game_lifecycle() {
        let mut game = Game::new();
        assert_eq!(game.phase, GamePhase::NotStarted);
        game.deploy_red(&builtin_aggressive()).unwrap();
        assert_eq!(game.phase, GamePhase::Deploying);
        game.deploy_blue(&builtin_defensive()).unwrap();
        assert_eq!(game.phase, GamePhase::Playing);
        assert_eq!(game.current_turn, Color::Red);
    }

    #[test]
    fn test_illegal_move_rejected() {
        let mut game = Game::new();
        game.deploy_red(&builtin_aggressive()).unwrap();
        game.deploy_blue(&builtin_defensive()).unwrap();
        let result = game.make_move(&Move::new(Position::new(6, 0), Position::new(6, 1)));
        assert!(result.is_err());
    }

    #[test]
    fn test_surrender() {
        let mut game = Game::new();
        game.deploy_red(&builtin_aggressive()).unwrap();
        game.deploy_blue(&builtin_defensive()).unwrap();
        let result = game.surrender();
        assert!(matches!(result, GameResult::BlueWins { .. }));
    }
}
