use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::board::Board;
use crate::piece::{CombatResult, Piece};
use crate::types::{Color, PieceKind, Position};

/// 一步走法，包含起点和终点位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    /// 起点位置
    pub from: Position,
    /// 终点位置
    pub to: Position,
}

impl Move {
    /// 创建一步新走法。
    ///
    /// # 参数
    /// - `from`: 起点位置
    /// - `to`: 终点位置
    ///
    /// # 返回值
    /// 包含起点和终点的 `Move` 实例
    pub fn new(from: Position, to: Position) -> Self { Move { from, to } }
}

/// 为指定颜色生成棋盘上所有合法走法。
///
/// 遍历棋盘上所有属于该颜色的棋子，为每枚可移动的棋子生成合法走法。
///
/// # 参数
/// - `board`: 当前棋盘状态
/// - `color`: 当前走子方的颜色
///
/// # 返回值
/// 所有合法走法的列表
pub fn generate_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    for r in 0..crate::board::ROWS {
        for c in 0..crate::board::COLS {
            let pos = Position::new(r as u8, c as u8);
            if let Some(p) = board.piece_at(pos) {
                if p.color == color { moves.extend(generate_piece_moves(board, pos, p)); }
            }
        }
    }
    moves
}

/// 为指定位置的单个棋子生成所有合法走法。
///
/// 走法生成规则：
/// - 不可移动的棋子（地雷、军旗）返回空列表
/// - 己方大本营内的棋子不可移动
/// - 已进入对方大本营的棋子不可继续移动
/// - 公路走法：1 步到邻接位置
/// - 铁路走法：
///   - 工兵：沿整个铁路网 BFS（可转弯）
///   - 非工兵：仅沿直线滑行（不可转弯），不限步数
///
/// # 参数
/// - `board`: 当前棋盘状态
/// - `from`: 棋子所在位置
/// - `piece`: 棋子引用
///
/// # 返回值
/// 该棋子的所有合法走法列表
pub fn generate_piece_moves(board: &Board, from: Position, piece: &Piece) -> Vec<Move> {
    if !piece.kind.can_move() { return vec![]; }

    if board.is_headquarters(from) && from.territory() == piece.color { return vec![]; }

    if board.is_enemy_hq(from, piece.color) { return vec![]; }

    let mut targets = HashSet::new();

    for nb in board.road_neighbors(from) {
        if can_enter(board, nb, piece.color) { targets.insert(nb); }
    }

    if board.is_rail_cell(from) {
        if piece.kind == PieceKind::GongBing {
            for dest in board.engineer_rail_slide(from, piece.color) {
                if can_enter(board, dest, piece.color) { targets.insert(dest); }
            }
        } else {
            for dest in board.straight_rail_slide(from, piece.color) {
                if can_enter(board, dest, piece.color) { targets.insert(dest); }
            }
        }
    }

    targets.into_iter().map(|to| Move::new(from, to)).collect()
}

/// 判断目标位置是否可以进入。
///
/// 规则：
/// - 不能进入己方大本营
/// - 不能进入有己方棋子的位置
/// - 不能进入有对方棋子保护的行营
///
/// # 参数
/// - `board`: 当前棋盘状态
/// - `dest`: 目标位置
/// - `color`: 走子方颜色
///
/// # 返回值
/// 如果可以进入则返回 `true`
fn can_enter(board: &Board, dest: Position, color: Color) -> bool {
    if board.is_headquarters(dest) && dest.territory() == color { return false; }
    if let Some(p) = board.piece_at(dest) {
        if p.color == color { return false; }
        if board.is_camp(dest) { return false; }
    }
    true
}

/// 判断一步走法是否合法。
///
/// # 参数
/// - `board`: 当前棋盘状态
/// - `mv`: 待判断的走法
/// - `color`: 走子方颜色
///
/// # 返回值
/// 如果该走法合法则返回 `true`
pub fn is_legal_move(board: &Board, mv: &Move, color: Color) -> bool {
    match board.piece_at(mv.from) {
        Some(p) if p.color == color && p.kind.can_move() => {
            if board.is_headquarters(mv.from) && mv.from.territory() == color { return false; }
            if board.is_enemy_hq(mv.from, color) { return false; }
            can_enter(board, mv.to, color) && generate_piece_moves(board, mv.from, p).contains(mv)
        }
        _ => false,
    }
}

/// 在棋盘上执行一步走法并返回结果。
///
/// 执行规则（CNVCS 暗棋）：
/// - 碰撞时双方棋子互相揭示身份
/// - 根据战斗结果处理棋子的移除/移动
/// - 若防守方为司令，揭示该方所有军旗位置
///
/// # 参数
/// - `board`: 可变引用的棋盘状态
/// - `mv`: 要执行的走法
///
/// # 返回值
/// 走法执行结果 `MoveResult`
pub fn execute_move(board: &mut Board, mv: &Move) -> MoveResult {
    let attacker = match board.piece_at(mv.from) { Some(p) => p.clone(), None => return MoveResult::Invalid };
    match board.piece_at(mv.to).cloned() {
        Some(defender) => {
            if let Some(p) = board.piece_at_mut(mv.from) { p.reveal(); }
            if let Some(p) = board.piece_at_mut(mv.to) { p.reveal(); }

            let result = match Piece::combat(&attacker, &defender) {
                CombatResult::Win => { board.move_piece(mv.from, mv.to); MoveResult::Capture { attacker: attacker.kind, defender: defender.kind } }
                CombatResult::Lose => { board.remove_piece(mv.from); MoveResult::Defeated { attacker: attacker.kind, defender: defender.kind } }
                CombatResult::Draw => { board.remove_piece(mv.from); board.remove_piece(mv.to); MoveResult::MutualDestruction { attacker: attacker.kind, defender: defender.kind } }
            };

            if defender.kind == PieceKind::SiLing {
                board.reveal_flags(defender.color);
            }
            result
        }
        None => {
            board.move_piece(mv.from, mv.to);
            MoveResult::Moved
        }
    }
}

/// 走法执行结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveResult {
    /// 普通移动（移动到空位）
    Moved,
    /// 吃子：攻击方吃掉了防守方
    Capture { attacker: PieceKind, defender: PieceKind },
    /// 被吃：攻击方被防守方吃掉
    Defeated { attacker: PieceKind, defender: PieceKind },
    /// 同归于尽：双方均被移除
    MutualDestruction { attacker: PieceKind, defender: PieceKind },
    /// 非法走法（起点无棋子等）
    Invalid,
}

impl MoveResult {
    /// 判断本次走法是否夺得了军旗。
    ///
    /// # 返回值
    /// 如果结果是吃子且被吃的是军旗则返回 `true`
    pub fn captured_flag(&self) -> bool { matches!(self, MoveResult::Capture { defender: PieceKind::JunQi, .. }) }

    /// 判断攻击方棋子是否在本次走法中存活。
    ///
    /// # 返回值
    /// 普通移动或吃子时返回 `true`；被吃或同归于尽时返回 `false`
    pub fn attacker_survived(&self) -> bool { matches!(self, MoveResult::Moved | MoveResult::Capture { .. }) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn place(b: &mut Board, row: u8, col: u8, kind: PieceKind, color: Color) {
        b.place_piece(Position::new(row, col), Piece::new(kind, color));
    }

    #[test]
    fn test_road_only_4dir() {
        let mut b = Board::new();
        place(&mut b, 3, 2, PieceKind::PaiZhang, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(3,2), b.piece_at(Position::new(3,2)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert_eq!(targets.len(), 8);
    }

    #[test]
    fn test_hq_piece_cannot_move() {
        let mut b = Board::new();
        place(&mut b, 0, 3, PieceKind::PaiZhang, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(0,3), b.piece_at(Position::new(0,3)).unwrap());
        assert!(moves.is_empty(), "己方大本营内棋子不可移动");
    }

    #[test]
    fn test_enemy_hq_piece_cannot_move() {
        let mut b = Board::new();
        place(&mut b, 11, 1, PieceKind::PaiZhang, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(11,1), b.piece_at(Position::new(11,1)).unwrap());
        assert!(moves.is_empty(), "CNVCS: 进入对方大本营后不可继续移动");
    }

    #[test]
    fn test_cannot_attack_into_camp() {
        let mut b = Board::new();
        place(&mut b, 6, 1, PieceKind::SiLing, Color::Red);
        place(&mut b, 7, 1, PieceKind::PaiZhang, Color::Blue);
        let moves = generate_piece_moves(&b, Position::new(6,1), b.piece_at(Position::new(6,1)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert!(!targets.contains(&Position::new(7,1)));
    }

    #[test]
    fn test_camp_piece_attacks_out() {
        let mut b = Board::new();
        place(&mut b, 4, 1, PieceKind::SiLing, Color::Red);
        place(&mut b, 5, 1, PieceKind::PaiZhang, Color::Blue);
        let moves = generate_piece_moves(&b, Position::new(4,1), b.piece_at(Position::new(4,1)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert!(targets.contains(&Position::new(5,1)));
    }

    #[test]
    fn test_cross_river_road() {
        let mut b = Board::new();
        place(&mut b, 5, 2, PieceKind::PaiZhang, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(5,2), b.piece_at(Position::new(5,2)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert!(targets.contains(&Position::new(6,2)), "公路连接前线");
    }

    #[test]
    fn test_bomb_rail_slide_straight() {
        let mut b = Board::new();
        place(&mut b, 5, 2, PieceKind::ZhaDan, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(5,2), b.piece_at(Position::new(5,2)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert!(targets.contains(&Position::new(5,0)), "横铁左滑");
        assert!(targets.contains(&Position::new(5,4)), "横铁右滑");
        assert!(targets.contains(&Position::new(6,2)), "纵铁下滑");
        assert!(!targets.contains(&Position::new(1,0)), "不可转弯");
    }

    #[test]
    fn test_engineer_bfs_on_rail() {
        let mut b = Board::new();
        place(&mut b, 1, 0, PieceKind::GongBing, Color::Red);
        let moves = generate_piece_moves(&b, Position::new(1,0), b.piece_at(Position::new(1,0)).unwrap());
        let targets: HashSet<Position> = moves.iter().map(|m| m.to).collect();
        assert!(targets.contains(&Position::new(1,4)), "沿行1横铁");
        assert!(targets.contains(&Position::new(10,0)), "沿列0纵铁");
        assert!(targets.contains(&Position::new(6,2)), "经列2纵铁可达");
    }
}
