use serde::{Deserialize, Serialize};
use crate::types::{Color, PieceKind};

/// 一枚具体的棋子，包含种类、颜色和揭示状态。
///
/// 在暗棋规则下，棋子的 `kind` 对对方不可见，
/// 仅在碰撞揭示或同色视角下可见。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    /// 棋子种类（如司令、炸弹等）
    pub kind: PieceKind,
    /// 棋子所属阵营颜色
    pub color: Color,
    /// 是否已揭示（碰撞后变为 true，对双方可见）
    pub revealed: bool,
}

impl Piece {
    /// 创建一枚新棋子，初始为未揭示状态。
    ///
    /// # 参数
    /// - `kind`: 棋子种类
    /// - `color`: 棋子所属阵营
    ///
    /// # 返回值
    /// 一枚未揭示的棋子
    pub fn new(kind: PieceKind, color: Color) -> Self {
        Piece { kind, color, revealed: false }
    }

    /// 揭示该棋子，将其 `revealed` 状态设为 `true`。
    ///
    /// 调用后该棋子的种类对双方玩家均可见。
    pub fn reveal(&mut self) {
        self.revealed = true;
    }

    /// 吃子判定：计算攻击方与防守方的战斗结果。
    ///
    /// 规则如下：
    /// - 炸弹：无论对方为何种棋子，结果均为同归于尽
    /// - 工兵 vs 地雷：工兵胜（工兵挖雷）
    /// - 非工兵 vs 地雷：攻击方死
    /// - 攻击军旗：攻击方胜
    /// - 同级对碰：同归于尽
    /// - 高级 vs 低级：高级胜（通过 `rank()` 比较）
    ///
    /// # 参数
    /// - `attacker`: 攻击方棋子
    /// - `defender`: 防守方棋子
    ///
    /// # 返回值
    /// 战斗结果：`Win`（攻击方胜）、`Lose`（攻击方败）、`Draw`（同归于尽）
    pub fn combat(attacker: &Piece, defender: &Piece) -> CombatResult {
        if attacker.kind == PieceKind::ZhaDan {
            return CombatResult::Draw;
        }
        if defender.kind == PieceKind::DiLei {
            return if attacker.kind == PieceKind::GongBing {
                CombatResult::Win
            } else {
                CombatResult::Lose
            };
        }
        if defender.kind == PieceKind::JunQi {
            return CombatResult::Win;
        }
        match attacker.kind.rank().cmp(&defender.kind.rank()) {
            std::cmp::Ordering::Greater => CombatResult::Win,
            std::cmp::Ordering::Less => CombatResult::Lose,
            std::cmp::Ordering::Equal => CombatResult::Draw,
        }
    }

    /// 返回指定视角下可看到的棋子种类。
    ///
    /// 暗棋规则下，只有己方棋子或被揭示后的棋子才能看到真实种类。
    /// 对方未揭示的棋子返回 `None`。
    ///
    /// # 参数
    /// - `viewer`: 观察者的颜色
    ///
    /// # 返回值
    /// 如果可视则返回 `Some(kind)`，否则返回 `None`
    pub fn visible_kind(&self, viewer: Color) -> Option<PieceKind> {
        if self.color == viewer || self.revealed { Some(self.kind) } else { None }
    }
}

/// 吃子结果
///
/// 表示一次碰撞战斗的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    /// 攻击方获胜（防守方被吃）
    Win,
    /// 攻击方失败（攻击方被吃）
    Lose,
    /// 同归于尽（双方均被移除）
    Draw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_higher_rank_wins() {
        let a = Piece::new(PieceKind::SiLing, Color::Red);
        let d = Piece::new(PieceKind::JunZhang, Color::Blue);
        assert_eq!(Piece::combat(&a, &d), CombatResult::Win);
    }

    #[test]
    fn test_lower_rank_loses() {
        let a = Piece::new(PieceKind::PaiZhang, Color::Red);
        let d = Piece::new(PieceKind::SiLing, Color::Blue);
        assert_eq!(Piece::combat(&a, &d), CombatResult::Lose);
    }

    #[test]
    fn test_equal_rank_draw() {
        let a = Piece::new(PieceKind::TuanZhang, Color::Red);
        let d = Piece::new(PieceKind::TuanZhang, Color::Blue);
        assert_eq!(Piece::combat(&a, &d), CombatResult::Draw);
    }

    #[test]
    fn test_bomb_draws_with_any() {
        let bomb = Piece::new(PieceKind::ZhaDan, Color::Red);
        let c = Piece::new(PieceKind::SiLing, Color::Blue);
        assert_eq!(Piece::combat(&bomb, &c), CombatResult::Draw);
    }

    #[test]
    fn test_engineer_defuses_mine() {
        let e = Piece::new(PieceKind::GongBing, Color::Red);
        let m = Piece::new(PieceKind::DiLei, Color::Blue);
        assert_eq!(Piece::combat(&e, &m), CombatResult::Win);
    }

    #[test]
    fn test_non_engineer_dies_to_mine() {
        let c = Piece::new(PieceKind::SiLing, Color::Red);
        let m = Piece::new(PieceKind::DiLei, Color::Blue);
        assert_eq!(Piece::combat(&c, &m), CombatResult::Lose);
    }

    #[test]
    fn test_capture_flag_wins() {
        let a = Piece::new(PieceKind::GongBing, Color::Red);
        let f = Piece::new(PieceKind::JunQi, Color::Blue);
        assert_eq!(Piece::combat(&a, &f), CombatResult::Win);
    }
}
