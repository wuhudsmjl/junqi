use serde::{Deserialize, Serialize};

/// 棋子种类 —— 共 12 种
///
/// 等级数字越大战斗力越强：
/// 司令(9) > 军长(8) > 师长(7) > 旅长(6) > 团长(5) > 营长(4) > 连长(3) > 排长(2) > 工兵(1)
///
/// 特殊棋子规则：
/// - 炸弹(ZhaDan)：与任何棋子碰撞时同归于尽
/// - 地雷(DiLei)：不可移动，仅工兵可挖，非工兵碰则攻击方死
/// - 军旗(JunQi)：不可移动，被对方夺走则判负
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceKind {
    /// 司令（最高统帅，rank=9）
    SiLing,
    /// 军长（rank=8）
    JunZhang,
    /// 师长（rank=7）
    ShiZhang,
    /// 旅长（rank=6）
    LvZhang,
    /// 团长（rank=5）
    TuanZhang,
    /// 营长（rank=4）
    YingZhang,
    /// 连长（rank=3）
    LianZhang,
    /// 排长（rank=2）
    PaiZhang,
    /// 工兵（rank=1，可挖地雷）
    GongBing,
    /// 炸弹（与任何棋子同归于尽，rank=0）
    ZhaDan,
    /// 地雷（不可移动，仅工兵可挖，rank=0）
    DiLei,
    /// 军旗（不可移动，被夺则输，rank=0）
    JunQi,
}

impl PieceKind {
    /// 获取该棋种的战力等级。
    ///
    /// 司令=9，军长=8，师长=7，旅长=6，团长=5，
    /// 营长=4，连长=3，排长=2，工兵=1，
    /// 炸弹、地雷、军旗返回 0（使用特殊规则判定胜负）。
    pub fn rank(self) -> u8 {
        match self {
            PieceKind::SiLing => 9,
            PieceKind::JunZhang => 8,
            PieceKind::ShiZhang => 7,
            PieceKind::LvZhang => 6,
            PieceKind::TuanZhang => 5,
            PieceKind::YingZhang => 4,
            PieceKind::LianZhang => 3,
            PieceKind::PaiZhang => 2,
            PieceKind::GongBing => 1,
            PieceKind::ZhaDan => 0,
            PieceKind::DiLei => 0,
            PieceKind::JunQi => 0,
        }
    }

    /// 获取每方拥有的该种棋子标准数量。
    ///
    /// 双方各有 25 枚棋子，具体分布为：
    /// 司令、军长、军旗各 1 枚；
    /// 师长、旅长、团长、营长、炸弹各 2 枚；
    /// 连长、排长、工兵、地雷各 3 枚。
    pub fn count_per_side(self) -> u8 {
        match self {
            PieceKind::SiLing => 1,
            PieceKind::JunZhang => 1,
            PieceKind::ShiZhang => 2,
            PieceKind::LvZhang => 2,
            PieceKind::TuanZhang => 2,
            PieceKind::YingZhang => 2,
            PieceKind::LianZhang => 3,
            PieceKind::PaiZhang => 3,
            PieceKind::GongBing => 3,
            PieceKind::ZhaDan => 2,
            PieceKind::DiLei => 3,
            PieceKind::JunQi => 1,
        }
    }

    /// 判断该棋种是否可以在棋盘上移动。
    ///
    /// 地雷和军旗不可移动，返回 `false`；其余棋种均可移动。
    pub fn can_move(self) -> bool {
        !matches!(self, PieceKind::DiLei | PieceKind::JunQi)
    }

    /// 返回该棋种的中文全称。
    pub fn chinese_name(self) -> &'static str {
        match self {
            PieceKind::SiLing => "司令",
            PieceKind::JunZhang => "军长",
            PieceKind::ShiZhang => "师长",
            PieceKind::LvZhang => "旅长",
            PieceKind::TuanZhang => "团长",
            PieceKind::YingZhang => "营长",
            PieceKind::LianZhang => "连长",
            PieceKind::PaiZhang => "排长",
            PieceKind::GongBing => "工兵",
            PieceKind::ZhaDan => "炸弹",
            PieceKind::DiLei => "地雷",
            PieceKind::JunQi => "军旗",
        }
    }

    /// 返回该棋种的中文单字缩写（用于棋盘显示）。
    pub fn short_name(self) -> &'static str {
        match self {
            PieceKind::SiLing => "司",
            PieceKind::JunZhang => "军",
            PieceKind::ShiZhang => "师",
            PieceKind::LvZhang => "旅",
            PieceKind::TuanZhang => "团",
            PieceKind::YingZhang => "营",
            PieceKind::LianZhang => "连",
            PieceKind::PaiZhang => "排",
            PieceKind::GongBing => "兵",
            PieceKind::ZhaDan => "弹",
            PieceKind::DiLei => "雷",
            PieceKind::JunQi => "旗",
        }
    }
}

/// 对战双方颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    /// 红方
    Red,
    /// 蓝方
    Blue,
}

impl Color {
    /// 返回对方颜色。
    ///
    /// 红方返回蓝方，蓝方返回红方。
    pub fn opponent(self) -> Color {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
        }
    }

    /// 返回该颜色的中文名称。
    pub fn chinese_name(self) -> &'static str {
        match self {
            Color::Red => "红方",
            Color::Blue => "蓝方",
        }
    }
}

/// 棋盘坐标：行 0~11，列 0~4
///
/// 坐标范围：行 0-5 为红方阵营（上半区），行 6-11 为蓝方阵营（下半区）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// 行坐标（0~11，0 为红方底线，11 为蓝方底线）
    pub row: u8,
    /// 列坐标（0~4，0 为最左列，4 为最右列）
    pub col: u8,
}

impl Position {
    /// 创建新的坐标位置。
    ///
    /// # 参数
    /// - `row`: 行坐标（有效范围 0~11）
    /// - `col`: 列坐标（有效范围 0~4）
    ///
    /// # 返回值
    /// 包含指定行和列的新 `Position` 实例
    pub fn new(row: u8, col: u8) -> Self {
        Position { row, col }
    }

    /// 判断该位置是否在棋盘有效范围内。
    ///
    /// 有效范围为行 0~11 且列 0~4。
    pub fn is_valid(self) -> bool {
        self.row < 12 && self.col < 5
    }

    /// 向指定方向移动一步，返回新位置。
    ///
    /// 如果移动后超出棋盘边界，则返回原位置不变。
    ///
    /// # 参数
    /// - `dir`: 移动方向
    ///
    /// # 返回值
    /// 移动一步后的新位置，若超出边界则返回原位置
    pub fn step(self, dir: Direction) -> Position {
        match dir {
            Direction::Up if self.row > 0 => Position::new(self.row - 1, self.col),
            Direction::Down if self.row < 11 => Position::new(self.row + 1, self.col),
            Direction::Left if self.col > 0 => Position::new(self.row, self.col - 1),
            Direction::Right if self.col < 4 => Position::new(self.row, self.col + 1),
            _ => self,
        }
    }

    /// 返回该位置所属的阵营颜色。
    ///
    /// 行 0-5 为红方阵营，行 6-11 为蓝方阵营。
    pub fn territory(self) -> Color {
        if self.row <= 5 { Color::Red } else { Color::Blue }
    }
}

/// 移动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 上（行号减小）
    Up,
    /// 下（行号增大）
    Down,
    /// 左（列号减小）
    Left,
    /// 右（列号增大）
    Right,
}

impl Direction {
    /// 返回所有方向的数组，按[上, 下, 左, 右]顺序排列。
    pub fn all() -> [Direction; 4] {
        [Direction::Up, Direction::Down, Direction::Left, Direction::Right]
    }
}

/// 棋盘格子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    /// 兵站（普通格子，可落子、可被攻击）
    Station,
    /// 行营（安全区，进入后不可被对方攻击）
    Camp,
    /// 大本营（军旗放置处，位于双方底线各两个）
    Headquarters,
}

impl CellType {
    /// 返回该格子类型的中文名称。
    pub fn chinese_name(self) -> &'static str {
        match self {
            CellType::Station => "兵站",
            CellType::Camp => "行营",
            CellType::Headquarters => "大本营",
        }
    }
}

/// AI 难度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiDifficulty {
    /// 中级（默认难度）
    #[default]
    Medium,
    /// 初级（最简单的 AI）
    Easy,
    /// 高级（较强 AI）
    Hard,
    /// 专家级（最强 AI）
    Expert,
}

impl AiDifficulty {
    /// 返回该难度等级的中文名称。
    pub fn chinese_name(self) -> &'static str {
        match self {
            AiDifficulty::Easy => "初级",
            AiDifficulty::Medium => "中级",
            AiDifficulty::Hard => "高级",
            AiDifficulty::Expert => "专家",
        }
    }

    /// 返回所有难度等级的数组，按[初级, 中级, 高级, 专家]顺序排列。
    pub fn all() -> [AiDifficulty; 4] {
        [AiDifficulty::Easy, AiDifficulty::Medium, AiDifficulty::Hard, AiDifficulty::Expert]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_rank_order() {
        assert!(PieceKind::SiLing.rank() > PieceKind::JunZhang.rank());
        assert!(PieceKind::JunZhang.rank() > PieceKind::ShiZhang.rank());
        assert!(PieceKind::ShiZhang.rank() > PieceKind::LvZhang.rank());
        assert!(PieceKind::LvZhang.rank() > PieceKind::TuanZhang.rank());
        assert!(PieceKind::TuanZhang.rank() > PieceKind::YingZhang.rank());
        assert!(PieceKind::YingZhang.rank() > PieceKind::LianZhang.rank());
        assert!(PieceKind::LianZhang.rank() > PieceKind::PaiZhang.rank());
        assert!(PieceKind::PaiZhang.rank() > PieceKind::GongBing.rank());
    }

    #[test]
    fn test_total_pieces_per_side() {
        let all = [
            PieceKind::SiLing, PieceKind::JunZhang, PieceKind::ShiZhang,
            PieceKind::LvZhang, PieceKind::TuanZhang, PieceKind::YingZhang,
            PieceKind::LianZhang, PieceKind::PaiZhang, PieceKind::GongBing,
            PieceKind::ZhaDan, PieceKind::DiLei, PieceKind::JunQi,
        ];
        let total: u8 = all.iter().map(|k| k.count_per_side()).sum();
        assert_eq!(total, 25);
    }

    #[test]
    fn test_position_valid() {
        assert!(Position::new(0, 0).is_valid());
        assert!(Position::new(11, 4).is_valid());
        assert!(!Position::new(12, 0).is_valid());
        assert!(!Position::new(0, 5).is_valid());
    }

    #[test]
    fn test_territory() {
        assert_eq!(Position::new(0, 0).territory(), Color::Red);
        assert_eq!(Position::new(5, 0).territory(), Color::Red);
        assert_eq!(Position::new(6, 0).territory(), Color::Blue);
        assert_eq!(Position::new(11, 0).territory(), Color::Blue);
    }

    #[test]
    fn test_opponent() {
        assert_eq!(Color::Red.opponent(), Color::Blue);
        assert_eq!(Color::Blue.opponent(), Color::Red);
    }
}
