use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::board::Board;
use crate::piece::Piece;
use crate::types::{Color, PieceKind, Position};

/// 布阵方案，定义每方 25 枚棋子的初始摆放位置。
///
/// CNVCS 布阵规则：
/// - 行营（10 个）内不可布子
/// - 军旗必须在大本营（两格之一）
/// - 地雷仅限后两排
/// - 炸弹不可放在前线（最后一行）
/// - 所有棋子必须分布在己方阵营内（行 0~5 或 行 6~11）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    /// 布阵名称
    pub name: String,
    /// 布阵描述
    pub description: String,
    /// 棋子列表，每项为 (种类, 位置)
    pub pieces: Vec<(PieceKind, Position)>,
    /// 布阵适用的阵营（`None` = 通用，可用于任何一方）
    #[serde(default)]
    pub color: Option<Color>,
}

impl Layout {
    /// 创建新的通用布阵方案（不指定阵营）。
    ///
    /// # 参数
    /// - `name`: 布阵名称
    /// - `description`: 布阵描述
    ///
    /// # 返回值
    /// 一个空的 `Layout` 实例（需通过 `add_piece` 添加棋子）
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Layout { name: name.into(), description: description.into(), pieces: Vec::with_capacity(25), color: None }
    }

    /// 创建指定阵营的布阵方案。
    ///
    /// # 参数
    /// - `name`: 布阵名称
    /// - `description`: 布阵描述
    /// - `color`: 指定阵营颜色
    ///
    /// # 返回值
    /// 一个指定阵营的空 `Layout` 实例
    pub fn new_for_color(name: impl Into<String>, description: impl Into<String>, color: Color) -> Self {
        Layout { name: name.into(), description: description.into(), pieces: Vec::with_capacity(25), color: Some(color) }
    }

    /// 向布阵中添加一枚棋子。
    ///
    /// # 参数
    /// - `kind`: 棋子种类
    /// - `pos`: 摆放位置（相对于红方视角的坐标，蓝方时会自动翻转）
    pub fn add_piece(&mut self, kind: PieceKind, pos: Position) { self.pieces.push((kind, pos)); }

    /// 将布阵方案应用到棋盘上。
    ///
    /// 如果应用于蓝方，棋盘上半区的坐标会自动翻转（行反射）。
    ///
    /// # 参数
    /// - `board`: 可变引用的棋盘
    /// - `color`: 应用该布阵的阵营颜色
    pub fn apply_to_board(&self, board: &mut Board, color: Color) {
        for &(kind, pos) in &self.pieces {
            let actual = match color {
                Color::Red => pos,
                Color::Blue if pos.row < 6 => Position::new(11 - pos.row, pos.col),
                _ => pos,
            };
            board.place_piece(actual, Piece::new(kind, color));
        }
    }

    fn to_actual_pos(pos: Position, color: Color) -> Position {
        match color {
            Color::Red => pos,
            Color::Blue if pos.row < 6 => Position::new(11 - pos.row, pos.col),
            _ => pos,
        }
    }

    /// 验证布阵方案是否合法。
    ///
    /// 检查项：
    /// - 棋子总数必须为 25
    /// - 每种棋子的数量必须符合标准
    /// - 军旗必须在大本营内
    /// - 地雷必须在后两排
    /// - 炸弹不能在前线
    /// - 行营内不能有棋子
    /// - 所有棋子必须在己方阵营内
    ///
    /// # 参数
    /// - `color`: 应用该布阵的阵营颜色
    ///
    /// # 返回值
    /// 成功返回 `Ok(())`，失败返回对应的 `LayoutError`
    pub fn validate(&self, color: Color) -> Result<(), LayoutError> {
        if self.pieces.len() != 25 {
            return Err(LayoutError::WrongPieceCount { expected: 25, actual: self.pieces.len() });
        }
        let mut counts: HashMap<PieceKind, u8> = HashMap::new();
        for &(kind, _) in &self.pieces { *counts.entry(kind).or_insert(0) += 1; }
        for kind in &ALL_KINDS {
            let expected = kind.count_per_side();
            if counts.get(kind).copied().unwrap_or(0) != expected {
                return Err(LayoutError::WrongKindCount { kind: *kind, expected, actual: counts.get(kind).copied().unwrap_or(0) });
            }
        }

        let hq_row = match color { Color::Red => 0, Color::Blue => 11 };
        for &(kind, pos) in &self.pieces {
            let actual = Self::to_actual_pos(pos, color);
            if kind == PieceKind::JunQi && (actual.row != hq_row || (actual.col != 1 && actual.col != 3)) {
                return Err(LayoutError::FlagNotInHeadquarters { pos: actual });
            }
        }

        let front_row = match color { Color::Red => 5, Color::Blue => 6 };
        let (back1, back2) = match color { Color::Red => (0, 1), Color::Blue => (11, 10) };
        for &(kind, pos) in &self.pieces {
            let actual = Self::to_actual_pos(pos, color);
            if kind == PieceKind::DiLei && actual.row != back1 && actual.row != back2 {
                return Err(LayoutError::MineNotInBackRows { pos: actual });
            }
            if kind == PieceKind::ZhaDan && actual.row == front_row {
                return Err(LayoutError::BombOnFrontLine { pos: actual });
            }
            if crate::board::Board::is_camp_position(actual) {
                return Err(LayoutError::PieceInCamp { pos: actual, kind });
            }
        }

        let (min_r, max_r) = match color { Color::Red => (0,5), Color::Blue => (6,11) };
        for &(_, pos) in &self.pieces {
            let actual = Self::to_actual_pos(pos, color);
            if actual.row < min_r || actual.row > max_r {
                return Err(LayoutError::PositionOutOfTerritory { pos: actual, color });
            }
        }
        Ok(())
    }

    /// 以红方视角验证布阵方案的合法性（通用验证）。
    ///
    /// # 返回值
    /// 成功返回 `Ok(())`，失败返回对应的 `LayoutError`
    pub fn validate_generic(&self) -> Result<(), LayoutError> { self.validate(Color::Red) }
}

const ALL_KINDS: [PieceKind; 12] = [
    PieceKind::SiLing, PieceKind::JunZhang, PieceKind::ShiZhang,
    PieceKind::LvZhang, PieceKind::TuanZhang, PieceKind::YingZhang,
    PieceKind::LianZhang, PieceKind::PaiZhang, PieceKind::GongBing,
    PieceKind::ZhaDan, PieceKind::DiLei, PieceKind::JunQi,
];

/// 布阵验证错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum LayoutError {
    /// 棋子总数错误
    #[error("棋子总数错误：期望 {expected}，实际 {actual}")]
    WrongPieceCount { expected: usize, actual: usize },
    /// 某种棋子的数量错误
    #[error("{kind:?} 数量错误：期望 {expected}，实际 {actual}")]
    WrongKindCount { kind: PieceKind, expected: u8, actual: u8 },
    /// 军旗不在大本营内
    #[error("军旗不在大本营")]
    FlagNotInHeadquarters { pos: Position },
    /// 地雷不在后两排
    #[error("地雷必须放在最后两排")]
    MineNotInBackRows { pos: Position },
    /// 炸弹放在前线
    #[error("炸弹不能放在前线")]
    BombOnFrontLine { pos: Position },
    /// 棋子放在行营内
    #[error("行营内不可布子: {kind:?}")]
    PieceInCamp { pos: Position, kind: PieceKind },
    /// 棋子位置超出阵营范围
    #[error("位置不在 {color:?} 阵营内")]
    PositionOutOfTerritory { pos: Position, color: Color },
}

/// 返回 5 套内置布阵方案。
///
/// 包含：猛攻型、防守型、均衡型、炸弹陷阱型、工兵突击型。
pub fn builtin_layouts() -> Vec<Layout> {
    vec![builtin_aggressive(), builtin_defensive(), builtin_balanced(), builtin_bomb_trap(), builtin_engineer_rush()]
}

/// 猛攻型：高级棋子前置快速进攻，后方相对空虚。
///
/// 前线集中了司令、军长、师长等高级棋子，适合主动进攻。
pub fn builtin_aggressive() -> Layout {
    let mut l = Layout::new("猛攻型", "高级棋子前置快速进攻，后方空虚");
    l.add_piece(PieceKind::SiLing, Position::new(5,2));
    l.add_piece(PieceKind::JunZhang, Position::new(5,0));
    l.add_piece(PieceKind::ShiZhang, Position::new(5,4));
    l.add_piece(PieceKind::ShiZhang, Position::new(5,1));
    l.add_piece(PieceKind::LvZhang, Position::new(5,3));
    l.add_piece(PieceKind::LvZhang, Position::new(4,2));
    l.add_piece(PieceKind::TuanZhang, Position::new(4,0));
    l.add_piece(PieceKind::TuanZhang, Position::new(4,4));
    l.add_piece(PieceKind::YingZhang, Position::new(3,0));
    l.add_piece(PieceKind::YingZhang, Position::new(3,4));
    l.add_piece(PieceKind::LianZhang, Position::new(3,1));
    l.add_piece(PieceKind::LianZhang, Position::new(3,3));
    l.add_piece(PieceKind::LianZhang, Position::new(2,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(2,0));
    l.add_piece(PieceKind::PaiZhang, Position::new(2,4));
    l.add_piece(PieceKind::GongBing, Position::new(1,0));
    l.add_piece(PieceKind::GongBing, Position::new(1,2));
    l.add_piece(PieceKind::GongBing, Position::new(1,4));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,3));
    l.add_piece(PieceKind::JunQi, Position::new(0,1));
    l.add_piece(PieceKind::PaiZhang, Position::new(0,3));
    l.add_piece(PieceKind::DiLei, Position::new(0,0));
    l.add_piece(PieceKind::DiLei, Position::new(0,2));
    l.add_piece(PieceKind::DiLei, Position::new(0,4));
    l
}

/// 防守型：高级棋子守护后方，前线以工兵和低阶棋子为主。
///
/// 军长、师长等高级棋子放在中后场，注重防守和反击。
pub fn builtin_defensive() -> Layout {
    let mut l = Layout::new("防守型", "高级棋子守护后方");
    l.add_piece(PieceKind::GongBing, Position::new(5,0));
    l.add_piece(PieceKind::GongBing, Position::new(5,2));
    l.add_piece(PieceKind::GongBing, Position::new(5,4));
    l.add_piece(PieceKind::PaiZhang, Position::new(5,1));
    l.add_piece(PieceKind::PaiZhang, Position::new(5,3));
    l.add_piece(PieceKind::LianZhang, Position::new(4,0));
    l.add_piece(PieceKind::LianZhang, Position::new(4,2));
    l.add_piece(PieceKind::LianZhang, Position::new(4,4));
    l.add_piece(PieceKind::YingZhang, Position::new(3,0));
    l.add_piece(PieceKind::YingZhang, Position::new(3,4));
    l.add_piece(PieceKind::ZhaDan, Position::new(3,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(3,3));
    l.add_piece(PieceKind::TuanZhang, Position::new(2,0));
    l.add_piece(PieceKind::TuanZhang, Position::new(2,4));
    l.add_piece(PieceKind::PaiZhang, Position::new(2,2));
    l.add_piece(PieceKind::LvZhang, Position::new(1,0));
    l.add_piece(PieceKind::LvZhang, Position::new(1,4));
    l.add_piece(PieceKind::ShiZhang, Position::new(1,1));
    l.add_piece(PieceKind::ShiZhang, Position::new(1,3));
    l.add_piece(PieceKind::JunZhang, Position::new(1,2));
    l.add_piece(PieceKind::JunQi, Position::new(0,1));
    l.add_piece(PieceKind::SiLing, Position::new(0,3));
    l.add_piece(PieceKind::DiLei, Position::new(0,0));
    l.add_piece(PieceKind::DiLei, Position::new(0,2));
    l.add_piece(PieceKind::DiLei, Position::new(0,4));
    l
}

/// 均衡型：攻守平衡的布阵方案。
///
/// 前线、中场、后方都有合理分布，适合各种局面。
pub fn builtin_balanced() -> Layout {
    let mut l = Layout::new("均衡型", "攻守平衡");
    l.add_piece(PieceKind::ShiZhang, Position::new(5,2));
    l.add_piece(PieceKind::TuanZhang, Position::new(5,0));
    l.add_piece(PieceKind::TuanZhang, Position::new(5,4));
    l.add_piece(PieceKind::GongBing, Position::new(5,1));
    l.add_piece(PieceKind::GongBing, Position::new(5,3));
    l.add_piece(PieceKind::LvZhang, Position::new(4,0));
    l.add_piece(PieceKind::LvZhang, Position::new(4,4));
    l.add_piece(PieceKind::LianZhang, Position::new(4,2));
    l.add_piece(PieceKind::JunZhang, Position::new(3,0));
    l.add_piece(PieceKind::ShiZhang, Position::new(3,4));
    l.add_piece(PieceKind::LianZhang, Position::new(3,1));
    l.add_piece(PieceKind::LianZhang, Position::new(3,3));
    l.add_piece(PieceKind::YingZhang, Position::new(2,0));
    l.add_piece(PieceKind::YingZhang, Position::new(2,4));
    l.add_piece(PieceKind::GongBing, Position::new(2,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,0));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,4));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,3));
    l.add_piece(PieceKind::JunQi, Position::new(0,1));
    l.add_piece(PieceKind::SiLing, Position::new(0,3));
    l.add_piece(PieceKind::DiLei, Position::new(0,0));
    l.add_piece(PieceKind::DiLei, Position::new(0,2));
    l.add_piece(PieceKind::DiLei, Position::new(0,4));
    l
}

/// 炸弹陷阱型：在关键位置布置炸弹，诱敌深入。
///
/// 利用炸弹防守关键通道，配合中高级棋子进行战术防守。
pub fn builtin_bomb_trap() -> Layout {
    let mut l = Layout::new("炸弹陷阱", "关键位置布置炸弹");
    l.add_piece(PieceKind::ShiZhang, Position::new(5,2));
    l.add_piece(PieceKind::LvZhang, Position::new(5,0));
    l.add_piece(PieceKind::LvZhang, Position::new(5,4));
    l.add_piece(PieceKind::TuanZhang, Position::new(5,1));
    l.add_piece(PieceKind::TuanZhang, Position::new(5,3));
    l.add_piece(PieceKind::GongBing, Position::new(4,0));
    l.add_piece(PieceKind::GongBing, Position::new(4,2));
    l.add_piece(PieceKind::GongBing, Position::new(4,4));
    l.add_piece(PieceKind::JunZhang, Position::new(3,0));
    l.add_piece(PieceKind::ShiZhang, Position::new(3,4));
    l.add_piece(PieceKind::YingZhang, Position::new(3,1));
    l.add_piece(PieceKind::YingZhang, Position::new(3,3));
    l.add_piece(PieceKind::LianZhang, Position::new(2,0));
    l.add_piece(PieceKind::LianZhang, Position::new(2,2));
    l.add_piece(PieceKind::LianZhang, Position::new(2,4));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,0));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,4));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,3));
    l.add_piece(PieceKind::JunQi, Position::new(0,1));
    l.add_piece(PieceKind::SiLing, Position::new(0,3));
    l.add_piece(PieceKind::DiLei, Position::new(0,0));
    l.add_piece(PieceKind::DiLei, Position::new(0,2));
    l.add_piece(PieceKind::DiLei, Position::new(0,4));
    l
}

/// 工兵突击型：工兵前置，快速挖雷夺旗。
///
/// 3 个工兵全部前线部署，配合高级棋子在后方支援，
/// 力求快速突破对方防线夺取军旗。
pub fn builtin_engineer_rush() -> Layout {
    let mut l = Layout::new("工兵突击", "工兵前置快速挖雷夺旗");
    l.add_piece(PieceKind::GongBing, Position::new(5,0));
    l.add_piece(PieceKind::GongBing, Position::new(5,2));
    l.add_piece(PieceKind::ShiZhang, Position::new(5,1));
    l.add_piece(PieceKind::GongBing, Position::new(5,3));
    l.add_piece(PieceKind::LvZhang, Position::new(5,4));
    l.add_piece(PieceKind::SiLing, Position::new(4,2));
    l.add_piece(PieceKind::JunZhang, Position::new(4,0));
    l.add_piece(PieceKind::ShiZhang, Position::new(4,4));
    l.add_piece(PieceKind::LvZhang, Position::new(3,0));
    l.add_piece(PieceKind::TuanZhang, Position::new(3,3));
    l.add_piece(PieceKind::TuanZhang, Position::new(3,4));
    l.add_piece(PieceKind::LianZhang, Position::new(3,1));
    l.add_piece(PieceKind::YingZhang, Position::new(2,0));
    l.add_piece(PieceKind::YingZhang, Position::new(2,4));
    l.add_piece(PieceKind::LianZhang, Position::new(2,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,0));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,2));
    l.add_piece(PieceKind::PaiZhang, Position::new(1,4));
    l.add_piece(PieceKind::LianZhang, Position::new(1,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(1,3));
    l.add_piece(PieceKind::JunQi, Position::new(0,1));
    l.add_piece(PieceKind::ZhaDan, Position::new(0,3));
    l.add_piece(PieceKind::DiLei, Position::new(0,0));
    l.add_piece(PieceKind::DiLei, Position::new(0,2));
    l.add_piece(PieceKind::DiLei, Position::new(0,4));
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_builtin_layouts_valid() {
        for layout in builtin_layouts() {
            assert!(layout.validate(Color::Red).is_ok(), "Layout {} failed", layout.name);
        }
    }

    #[test]
    fn test_flag_not_in_hq() {
        let mut l = Layout::new("test", "");
        l.add_piece(PieceKind::JunQi, Position::new(3,2));
        fill_remaining(&mut l);
        assert!(l.validate(Color::Red).is_err());
    }

    fn fill_remaining(layout: &mut Layout) {
        let mut counts: HashMap<PieceKind, u8> = HashMap::new();
        for &(k, _) in &layout.pieces { *counts.entry(k).or_insert(0) += 1; }
        let positions: Vec<Position> = (0..6u8).flat_map(|r| (0..5u8).map(move |c| Position::new(r,c)))
            .filter(|p| !crate::board::Board::is_camp_position(*p)).collect();
        let mut idx = 0;
        for kind in &ALL_KINDS {
            let need = kind.count_per_side();
            let have = counts.get(kind).copied().unwrap_or(0);
            for _ in have..need {
                if idx < positions.len() {
                    layout.add_piece(*kind, positions[idx]);
                    idx += 1;
                }
            }
        }
    }
}
