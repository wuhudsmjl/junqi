use serde::{Deserialize, Serialize};
use crate::game::GameResult;
use crate::layout::Layout;
use crate::moves::{Move, MoveResult};
use crate::types::Color;

/// 一步对局记录，用于复盘回放。
///
/// 包含步数、走法、结果和走子方信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// 步数编号（从 0 开始）
    pub step_number: u32,
    /// 走的棋步（起点和终点）
    pub mv: Move,
    /// 走法执行结果
    pub result: MoveResult,
    /// 执行该步的一方颜色
    pub turn: Color,
}

/// 复盘数据，包含完整对局信息。
///
/// 用于对局的保存、加载和回放。
/// 包含双方玩家名称、布阵方案、走法记录和结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    /// 红方玩家名称
    pub red_name: String,
    /// 蓝方玩家名称
    pub blue_name: String,
    /// 红方布阵方案
    pub red_layout: Option<Layout>,
    /// 蓝方布阵方案
    pub blue_layout: Option<Layout>,
    /// 走法步骤历史记录
    pub steps: Vec<Step>,
    /// 对局结果
    pub result: Option<GameResult>,
    /// 是否已保存（`false` 表示滑动窗口中的临时数据，可能被自动清理）
    #[serde(default)]
    pub saved: bool,
}

impl Replay {
    /// 创建新的复盘数据。
    ///
    /// # 参数
    /// - `red_name`: 红方玩家名称
    /// - `blue_name`: 蓝方玩家名称
    ///
    /// # 返回值
    /// 空的 `Replay` 实例（尚无布阵和走法记录）
    pub fn new(red_name: &str, blue_name: &str) -> Self {
        Replay {
            red_name: red_name.to_string(),
            blue_name: blue_name.to_string(),
            red_layout: None,
            blue_layout: None,
            steps: Vec::new(),
            result: None,
            saved: false,
        }
    }

    /// 获取总步数。
    ///
    /// # 返回值
    /// 已记录的对局步数
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// 获取指定索引的步骤记录。
    ///
    /// # 参数
    /// - `n`: 步骤索引（从 0 开始）
    ///
    /// # 返回值
    /// 如果索引有效则返回 `Some(&Step)`，否则返回 `None`
    pub fn step(&self, n: usize) -> Option<&Step> {
        self.steps.get(n)
    }

    /// 从指定视角查看复盘步骤（CNVCS 视角信息过滤）。
    ///
    /// # 参数
    /// - `n`: 步骤索引（从 0 开始）
    /// - `_viewer`: 观察者视角颜色（预留，当前未过滤）
    ///
    /// # 返回值
    /// 如果索引有效则返回 `Some(StepView)`，否则返回 `None`
    pub fn step_from_perspective(&self, n: usize, _viewer: Color) -> Option<StepView> {
        let step = self.steps.get(n)?;
        Some(StepView {
            step_number: step.step_number,
            mv: step.mv,
            result_visible: true,
            turn: step.turn,
        })
    }

    /// 获取对局结果的中文描述。
    ///
    /// # 返回值
    /// 格式化的结果字符串（如 "红方(玩家A)获胜 - 夺得军旗"）
    pub fn result_description(&self) -> String {
        match &self.result {
            Some(result) => match result {
                GameResult::RedWins { reason } => {
                    format!("红方({})获胜 - {}", self.red_name, reason_desc(reason))
                }
                GameResult::BlueWins { reason } => {
                    format!("蓝方({})获胜 - {}", self.blue_name, reason_desc(reason))
                }
                GameResult::Draw { reason } => {
                    format!("平局 - {}", draw_reason_desc(reason))
                }
            },
            None => "对局未结束".to_string(),
        }
    }
}

/// 复盘步骤视图（从指定视角观察的单步信息）
#[derive(Debug, Clone)]
pub struct StepView {
    /// 步数编号
    pub step_number: u32,
    /// 走的棋步
    pub mv: Move,
    /// 结果是否可见（暗棋视角下可能为假）
    pub result_visible: bool,
    /// 执行该步的一方颜色
    pub turn: Color,
}

fn reason_desc(reason: &crate::game::WinReason) -> &'static str {
    match reason {
        crate::game::WinReason::CapturedFlag => "夺得军旗",
        crate::game::WinReason::NoMovesLeft => "对方无棋可走",
        crate::game::WinReason::Surrender => "对方认输",
        crate::game::WinReason::Timeout => "对方超时",
    }
}

fn draw_reason_desc(reason: &crate::game::DrawReason) -> &'static str {
    match reason {
        crate::game::DrawReason::Agreed => "双方同意和棋",
        crate::game::DrawReason::NoAttackPieces => "双方均无进攻棋子",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_creation() {
        let replay = Replay::new("玩家A", "玩家B");
        assert_eq!(replay.red_name, "玩家A");
        assert_eq!(replay.blue_name, "玩家B");
        assert_eq!(replay.total_steps(), 0);
    }
}
