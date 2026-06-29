use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::piece::Piece;
use crate::types::{CellType, Color, Position};

pub const ROWS: usize = 12;
pub const COLS: usize = 5;

/// 行营位置列表（全部10个）
///
/// 每方5个行营，红方在行2-4，蓝方在行7-9。
pub const CAMP_POSITIONS: [(u8,u8); 10] = [
    (2,1),(2,3),(3,2),(4,1),(4,3),
    (7,1),(7,3),(8,2),(9,1),(9,3),
];

/// 棋盘 —— 12行×5列的军棋标准棋盘
///
/// 棋盘采用 12×5 网格，行 0~11，列 0~4。
/// 行 0~5 为红方阵营，行 6~11 为蓝方阵营，中间行 5~6 为前线交界。
/// 棋盘包含公路和铁路两种连通方式，以及兵站、行营、大本营三种格子类型。
///
/// # 公路线规则
/// - 横向：第 0,2,3,4,7,8,9,11 行所有列之间有公路连接
/// - 纵向：所有列的行 0-1 和 10-11；第 1,2,3 列的行 1-2,2-3,3-4,4-5,6-7,7-8,8-9,9-10
/// - 斜向：每个行营与左上、右上、左下、右下 4 个方向各有 1 条公路
///
/// # 铁路线规则
/// - 横向：第 1,5,6,10 行整行
/// - 纵向：第 0,4 列的行 1-10；第 2 列的行 5-6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    cells: [[Option<Piece>; COLS]; ROWS],
    cell_types: [[CellType; COLS]; ROWS],
    road_adj: Vec<Vec<Vec<Position>>>,
    rail_adj: Vec<Vec<Vec<Position>>>,
}

impl Board {
    /// 创建一个空棋盘，所有格子初始无棋子。
    ///
    /// 会同时构建公路和铁路的邻接表。
    pub fn new() -> Self {
        let cells = [[None; COLS]; ROWS];
        let cell_types = Self::build_cell_types();
        let road_adj = Self::build_road();
        let rail_adj = Self::build_rail();
        Board { cells, cell_types, road_adj, rail_adj }
    }

    fn build_cell_types() -> [[CellType; COLS]; ROWS] {
        let mut t = [[CellType::Station; COLS]; ROWS];
        t[0][1] = CellType::Headquarters; t[0][3] = CellType::Headquarters;
        t[11][1] = CellType::Headquarters; t[11][3] = CellType::Headquarters;
        for &(r,c) in &CAMP_POSITIONS { t[r as usize][c as usize] = CellType::Camp; }
        t
    }

    fn build_road() -> Vec<Vec<Vec<Position>>> {
        let mut adj: Vec<Vec<HashSet<Position>>> = vec![vec![HashSet::new(); COLS]; ROWS];

        let mut link = |a: Position, b: Position| {
            if a.is_valid() && b.is_valid() {
                adj[a.row as usize][a.col as usize].insert(b);
                adj[b.row as usize][b.col as usize].insert(a);
            }
        };

        for row in [0u8, 2, 3, 4, 7, 8, 9, 11] {
            for col in 0..(COLS as u8 - 1) {
                link(Position::new(row, col), Position::new(row, col + 1));
            }
        }

        for col in 0..COLS as u8 {
            link(Position::new(0, col), Position::new(1, col));
            link(Position::new(10, col), Position::new(11, col));
        }

        for col in [1u8, 2, 3] {
            for r in [1u8,2,3,4,6,7,8,9] {
                if r < 11 { link(Position::new(r, col), Position::new(r + 1, col)); }
            }
        }

        let dirs: [(i8,i8); 4] = [(-1,-1), (-1,1), (1,-1), (1,1)];
        for &(cr, cc) in &CAMP_POSITIONS {
            let camp = Position::new(cr, cc);
            for (dr, dc) in &dirs {
                let nr = cr as i8 + dr;
                let nc = cc as i8 + dc;
                if nr >= 0 && nr < ROWS as i8 && nc >= 0 && nc < COLS as i8 {
                    link(camp, Position::new(nr as u8, nc as u8));
                }
            }
        }

        adj.into_iter().map(|row| row.into_iter().map(|set| {
            let mut v: Vec<Position> = set.into_iter().collect();
            v.sort_by_key(|p| (p.row, p.col));
            v
        }).collect()).collect()
    }

    fn build_rail() -> Vec<Vec<Vec<Position>>> {
        let mut adj: Vec<Vec<HashSet<Position>>> = vec![vec![HashSet::new(); COLS]; ROWS];

        let mut link = |a: Position, b: Position| {
            if a.is_valid() && b.is_valid() {
                adj[a.row as usize][a.col as usize].insert(b);
                adj[b.row as usize][b.col as usize].insert(a);
            }
        };

        for row in [1u8, 5, 6, 10] {
            for col in 0..(COLS as u8 - 1) {
                link(Position::new(row, col), Position::new(row, col + 1));
            }
        }

        for col in [0u8, 4] {
            for r in 1u8..10u8 {
                link(Position::new(r, col), Position::new(r + 1, col));
            }
        }

        link(Position::new(5, 2), Position::new(6, 2));

        adj.into_iter().map(|row| row.into_iter().map(|set| {
            let mut v: Vec<Position> = set.into_iter().collect();
            v.sort_by_key(|p| (p.row, p.col));
            v
        }).collect()).collect()
    }

    /// 获取指定位置的格子类型。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 该位置的 `CellType`（兵站、行营或大本营）
    pub fn cell_type(&self, pos: Position) -> CellType { self.cell_types[pos.row as usize][pos.col as usize] }

    /// 获取指定位置的棋子引用（不可变）。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 如果有棋子则返回 `Some(&Piece)`，否则返回 `None`
    pub fn piece_at(&self, pos: Position) -> Option<&Piece> { self.cells[pos.row as usize][pos.col as usize].as_ref() }

    /// 获取指定位置的棋子可变引用。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 如果有棋子则返回 `Some(&mut Piece)`，否则返回 `None`
    pub fn piece_at_mut(&mut self, pos: Position) -> Option<&mut Piece> { self.cells[pos.row as usize][pos.col as usize].as_mut() }

    /// 在指定位置放置一枚棋子。
    ///
    /// # 参数
    /// - `pos`: 放置位置
    /// - `piece`: 要放置的棋子
    pub fn place_piece(&mut self, pos: Position, piece: Piece) { self.cells[pos.row as usize][pos.col as usize] = Some(piece); }

    /// 移除指定位置的棋子并返回它。
    ///
    /// # 参数
    /// - `pos`: 要移除棋子的位置
    ///
    /// # 返回值
    /// 被移除的棋子，如果该位置无棋子则返回 `None`
    pub fn remove_piece(&mut self, pos: Position) -> Option<Piece> { self.cells[pos.row as usize][pos.col as usize].take() }

    /// 将一枚棋子从 `from` 位置移动到 `to` 位置。
    ///
    /// 如果 `from` 位置无棋子，则不做任何操作。
    ///
    /// # 参数
    /// - `from`: 起点位置
    /// - `to`: 终点位置
    pub fn move_piece(&mut self, from: Position, to: Position) {
        let p = self.remove_piece(from);
        if let Some(p) = p { self.place_piece(to, p); }
    }

    /// 获取指定位置的所有公路邻接位置（1步可达）。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 公路可达的邻接位置列表
    pub fn road_neighbors(&self, pos: Position) -> Vec<Position> {
        self.road_adj[pos.row as usize][pos.col as usize].clone()
    }

    /// 获取指定位置的所有铁路邻接位置（1步可达）。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 铁路直接邻接的位置列表
    pub fn rail_neighbors(&self, pos: Position) -> Vec<Position> {
        self.rail_adj[pos.row as usize][pos.col as usize].clone()
    }

    /// 判断指定位置是否位于铁路格上（需要 Board 实例）。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 如果该格有铁路连接则返回 `true`
    pub fn is_rail_cell(&self, pos: Position) -> bool {
        !self.rail_adj[pos.row as usize][pos.col as usize].is_empty()
    }

    /// 判断任意位置是否是铁路格（无需 Board 实例，纯位置判断）。
    ///
    /// 规则：
    /// - 列 0 和列 4 的行 1~10 为纵向铁路
    /// - 行 1、5、6、10 的所有列为横向铁路
    /// - 列 2 的行 5~6 为纵向铁路
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 如果该位置是铁路格则返回 `true`
    pub fn is_rail_cell_position(pos: Position) -> bool {
        let row = pos.row as usize;
        let col = pos.col as usize;
        if col == 0 || col == 4 {
            return (1..=10).contains(&row);
        }
        if row == 1 || row == 5 || row == 6 || row == 10 {
            return true;
        }
        if col == 2 && (row == 5 || row == 6) {
            return true;
        }
        false
    }

    /// 工兵铁路滑行：使用广度优先搜索沿整个铁路网移动（可任意转弯）。
    ///
    /// 工兵在铁路上可以沿任何方向连续滑行，
    /// 遇到己方棋子停止（不可经过），遇到敌方棋子可攻击（作为终点）。
    ///
    /// # 参数
    /// - `from`: 起始位置
    /// - `color`: 工兵所属颜色
    ///
    /// # 返回值
    /// 工兵可以到达的所有位置列表（含敌方棋子所在位置）
    pub fn engineer_rail_slide(&self, from: Position, color: Color) -> Vec<Position> {
        let mut visited = vec![vec![false; COLS]; ROWS];
        let mut result = Vec::new();
        let mut stack = vec![from];
        visited[from.row as usize][from.col as usize] = true;
        while let Some(cur) = stack.pop() {
            for &nb in &self.rail_adj[cur.row as usize][cur.col as usize] {
                if visited[nb.row as usize][nb.col as usize] { continue; }
                visited[nb.row as usize][nb.col as usize] = true;
                if let Some(p) = self.piece_at(nb) {
                    if p.color == color { continue; }
                    result.push(nb);
                } else {
                    result.push(nb);
                    stack.push(nb);
                }
            }
        }
        result
    }

    /// 非工兵铁路滑行：仅沿直线（横向或纵向）滑行，不限步数，不可转弯。
    ///
    /// 非工兵棋子（如炸弹、师长等）在铁路上只能沿一个方向直行，
    /// 遇到己方棋子则停止（该方向不可达），遇到敌方棋子可攻击。
    /// 每一步都通过铁路邻接表验证连通性。
    ///
    /// # 参数
    /// - `from`: 起始位置
    /// - `color`: 棋子所属颜色
    ///
    /// # 返回值
    /// 沿直线铁路可达的所有位置列表（含敌方棋子所在位置）
    pub fn straight_rail_slide(&self, from: Position, color: Color) -> Vec<Position> {
        let mut result = Vec::new();
        let dirs: [(i8,i8); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
        for (dr, dc) in &dirs {
            let mut prev = from;
            loop {
                let nr = prev.row as i8 + dr;
                let nc = prev.col as i8 + dc;
                if nr < 0 || nr >= ROWS as i8 || nc < 0 || nc >= COLS as i8 { break; }
                let pos = Position::new(nr as u8, nc as u8);
                if !self.rail_adj[prev.row as usize][prev.col as usize].contains(&pos) { break; }
                if let Some(p) = self.piece_at(pos) {
                    if p.color == color { break; }
                    result.push(pos);
                    break;
                }
                result.push(pos);
                prev = pos;
            }
        }
        result
    }

    /// 判断指定位置是否是行营。
    pub fn is_camp(&self, pos: Position) -> bool { self.cell_type(pos) == CellType::Camp }

    /// 判断指定位置是否是大本营。
    pub fn is_headquarters(&self, pos: Position) -> bool { self.cell_type(pos) == CellType::Headquarters }

    /// 判断任意位置是否是行营（无需 Board 实例）。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    ///
    /// # 返回值
    /// 如果该位置是行营则返回 `true`
    pub fn is_camp_position(pos: Position) -> bool {
        CAMP_POSITIONS.contains(&(pos.row, pos.col))
    }

    /// 判断指定位置是否是对方的⼤本营。
    ///
    /// # 参数
    /// - `pos`: 棋盘坐标
    /// - `my_color`: 己方颜色
    ///
    /// # 返回值
    /// 如果该位置是对方的大本营则返回 `true`
    pub fn is_enemy_hq(&self, pos: Position, my_color: Color) -> bool {
        self.is_headquarters(pos) && pos.territory() != my_color
    }

    /// 司令阵亡时揭示该方所有军旗位置。
    ///
    /// 根据 CNVCS 规则，当司令被吃时，该方所有大本营中的军旗会被揭示，
    /// 对双方可见。
    ///
    /// # 参数
    /// - `color`: 司令所属的颜色（即被揭示军旗的一方）
    pub fn reveal_flags(&mut self, color: Color) {
        for pos in self.headquarters(color) {
            if let Some(p) = self.piece_at_mut(pos) {
                if p.kind == crate::types::PieceKind::JunQi { p.reveal(); }
            }
        }
    }

    /// 获取指定颜色的大本营位置列表。
    ///
    /// # 参数
    /// - `color`: 阵营颜色
    ///
    /// # 返回值
    /// 包含两个大本营坐标的向量，红方为 (0,1) 和 (0,3)，蓝方为 (11,1) 和 (11,3)
    pub fn headquarters(&self, color: Color) -> Vec<Position> {
        match color { Color::Red=>vec![Position::new(0,1),Position::new(0,3)], Color::Blue=>vec![Position::new(11,1),Position::new(11,3)] }
    }

    /// 获取指定颜色阵营的所有格子位置。
    ///
    /// # 参数
    /// - `color`: 阵营颜色
    ///
    /// # 返回值
    /// 该阵营所有格子的位置列表（红方行 0~5，蓝方行 6~11）
    pub fn territory_positions(&self, color: Color) -> Vec<Position> {
        let (s,e)=match color{Color::Red=>(0,5),Color::Blue=>(6,11)};
        (s..=e).flat_map(|r|(0..COLS as u8).map(move|c|Position::new(r,c))).collect()
    }

    /// 清空棋盘上所有棋子。
    pub fn clear_pieces(&mut self) { self.cells = [[None; COLS]; ROWS]; }

    /// 遍历棋盘上所有位置及其棋子。
    ///
    /// # 返回值
    /// 一个迭代器，生成 `(Position, Option<&Piece>)` 元组
    pub fn iter(&self) -> impl Iterator<Item=(Position,Option<&Piece>)> {
        self.cells.iter().enumerate().flat_map(|(r,row)|row.iter().enumerate().map(move|(c,cell)|(Position::new(r as u8,c as u8),cell.as_ref())))
    }
}

impl Default for Board { fn default()->Self{Self::new()} }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_horizontal_rows() {
        let b = Board::new();
        for row in [0u8,2,3,4,7,8,9,11] {
            assert!(b.road_neighbors(Position::new(row,0)).contains(&Position::new(row,1)));
            assert!(b.road_neighbors(Position::new(row,3)).contains(&Position::new(row,4)));
        }
        for row in [1u8,5,6,10] {
            let ns = b.road_neighbors(Position::new(row, 2));
            let has_horiz = ns.iter().any(|p| p.row == row && p.col == 1);
            assert!(!has_horiz, "行{}不应有横向公路", row);
        }
    }

    #[test]
    fn test_road_diagonal_from_camps() {
        let b = Board::new();
        let ns = b.road_neighbors(Position::new(2, 1));
        assert!(ns.contains(&Position::new(1,0)));
        assert!(ns.contains(&Position::new(1,2)));
        assert!(ns.contains(&Position::new(3,0)));
        assert!(ns.contains(&Position::new(3,2)));
    }

    #[test]
    fn test_rail_horizontal() {
        let b = Board::new();
        for row in [1u8,5,6,10] {
            assert!(b.rail_neighbors(Position::new(row,0)).contains(&Position::new(row,1)));
            assert!(b.rail_neighbors(Position::new(row,3)).contains(&Position::new(row,4)));
        }
    }

    #[test]
    fn test_rail_vertical() {
        let b = Board::new();
        assert!(b.rail_neighbors(Position::new(1,0)).contains(&Position::new(2,0)));
        assert!(b.rail_neighbors(Position::new(5,0)).contains(&Position::new(6,0)));
        assert!(b.rail_neighbors(Position::new(9,4)).contains(&Position::new(10,4)));
        assert!(b.rail_neighbors(Position::new(5,2)).contains(&Position::new(6,2)));
    }

    #[test]
    fn test_gap_at_row56_col1_col3() {
        let b = Board::new();
        assert!(b.road_neighbors(Position::new(5,1)).iter().all(|p| *p != Position::new(6,1)));
        assert!(b.rail_neighbors(Position::new(5,1)).iter().all(|p| *p != Position::new(6,1)));
        assert!(b.road_neighbors(Position::new(6,1)).iter().all(|p| *p != Position::new(5,1)));
        assert!(b.rail_neighbors(Position::new(6,1)).iter().all(|p| *p != Position::new(5,1)));
        assert!(b.road_neighbors(Position::new(5,3)).iter().all(|p| *p != Position::new(6,3)));
        assert!(b.rail_neighbors(Position::new(5,3)).iter().all(|p| *p != Position::new(6,3)));
        assert!(b.rail_neighbors(Position::new(5,0)).contains(&Position::new(6,0)));
        assert!(b.rail_neighbors(Position::new(5,2)).contains(&Position::new(6,2)));
        assert!(b.rail_neighbors(Position::new(5,4)).contains(&Position::new(6,4)));
    }

    #[test]
    fn test_no_diagonal_rail() {
        let b = Board::new();
        assert!(!b.rail_neighbors(Position::new(0,0)).contains(&Position::new(1,1)));
        assert!(!b.rail_neighbors(Position::new(0,4)).contains(&Position::new(1,3)));
    }
}
