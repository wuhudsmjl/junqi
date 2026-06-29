use junqi_core::board::Board;
use junqi_core::moves::generate_moves;
use junqi_core::types::{Color, PieceKind, Position};

/// 棋子基础面值
fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::SiLing => 1000,
        PieceKind::JunZhang => 800,
        PieceKind::ShiZhang => 600,
        PieceKind::LvZhang => 400,
        PieceKind::TuanZhang => 300,
        PieceKind::YingZhang => 200,
        PieceKind::LianZhang => 150,
        PieceKind::PaiZhang => 100,
        PieceKind::GongBing => 250,
        PieceKind::ZhaDan => 350,
        PieceKind::DiLei => 200,
        PieceKind::JunQi => 10000,
    }
}

/// 位置加分——根据棋子在棋盘上的位置给予加成
fn position_bonus(kind: PieceKind, pos: Position, color: Color) -> i32 {
    let mut bonus = 0;

    if junqi_core::board::Board::is_camp_position(pos) {
        bonus += 30;
    }

    if kind == PieceKind::GongBing && junqi_core::board::Board::is_rail_cell_position(pos) {
        bonus += 20;
    }

    if kind == PieceKind::DiLei {
        if dist_to_hq(pos, color) <= 1 {
            bonus += 50;
        }
    }

    if kind.rank() >= 6 && dist_to_hq(pos, color) <= 2 {
        bonus += 15;
    }

    let front_row = match color {
        Color::Red => 5,
        Color::Blue => 6,
    };
    if pos.row == front_row && kind.can_move() && kind.rank() >= 3 {
        bonus += 25;
    }

    bonus
}

/// 到大本营的距离
fn dist_to_hq(pos: Position, color: Color) -> u8 {
    let hq_positions = match color {
        Color::Red => [(0u8, 1u8), (0, 3)],
        Color::Blue => [(11, 1), (11, 3)],
    };
    let d1 = (pos.row as i16 - hq_positions[0].0 as i16).unsigned_abs() as u8
        + (pos.col as i16 - hq_positions[0].1 as i16).unsigned_abs() as u8;
    let d2 = (pos.row as i16 - hq_positions[1].0 as i16).unsigned_abs() as u8
        + (pos.col as i16 - hq_positions[1].1 as i16).unsigned_abs() as u8;
    d1.min(d2)
}

/// 评估棋盘局面
///
/// 返回正数表示红方有利，负数表示蓝方有利。
///
/// # 参数
/// - `board`: 当前棋盘状态
/// - `color`: 评估视角（通常传入 AI 控制的颜色，返回值为正即对 AI 有利）
pub fn evaluate(board: &Board, color: Color) -> i32 {
    let mut score: i32 = 0;

    for row in 0..junqi_core::board::ROWS {
        for col in 0..junqi_core::board::COLS {
            let pos = Position::new(row as u8, col as u8);
            if let Some(piece) = board.piece_at(pos) {
                let value = piece_value(piece.kind);
                let bonus = position_bonus(piece.kind, pos, piece.color);

                if piece.color == Color::Red {
                    score += value + bonus;
                } else {
                    score -= value + bonus;
                }
            }
        }
    }

    let red_moves = count_moves(board, Color::Red);
    let blue_moves = count_moves(board, Color::Blue);
    score += (red_moves as i32 - blue_moves as i32) * 5;

    score += threat_assessment(board);

    match color {
        Color::Red => score,
        Color::Blue => -score,
    }
}

/// 统计某方可走步数（机动性）
fn count_moves(board: &Board, color: Color) -> usize {
    generate_moves(board, color).len()
}

/// 威胁度评估
///
/// 检测双方高级棋子之间的威胁关系。
fn threat_assessment(board: &Board) -> i32 {
    let mut threat = 0;

    for row1 in 0..junqi_core::board::ROWS {
        for col1 in 0..junqi_core::board::COLS {
            let pos1 = Position::new(row1 as u8, col1 as u8);
            if let Some(p1) = board.piece_at(pos1) {
                if !p1.kind.can_move() {
                    continue;
                }

                for row2 in 0..junqi_core::board::ROWS {
                    for col2 in 0..junqi_core::board::COLS {
                        let pos2 = Position::new(row2 as u8, col2 as u8);
                        if let Some(p2) = board.piece_at(pos2) {
                            if p1.color == p2.color {
                                continue;
                            }
                            let dist = (pos1.row as i16 - pos2.row as i16).unsigned_abs()
                                + (pos1.col as i16 - pos2.col as i16).unsigned_abs();
                            if dist <= 1 || (p1.kind == PieceKind::GongBing && junqi_core::board::Board::is_rail_cell_position(pos1) && junqi_core::board::Board::is_rail_cell_position(pos2)) {
                                if p1.kind.rank() > p2.kind.rank()
                                    || p1.kind == PieceKind::ZhaDan
                                    || (p1.kind == PieceKind::GongBing && p2.kind == PieceKind::DiLei)
                                {
                                    let t = piece_value(p2.kind) / 4;
                                    if p1.color == Color::Red {
                                        threat += t;
                                    } else {
                                        threat -= t;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    threat
}

#[cfg(test)]
mod tests {
    use super::*;
    use junqi_core::game::Game;
    use junqi_core::layout::builtin_balanced;

    #[test]
    fn test_evaluate_balanced_start() {
        let mut game = Game::new();
        game.deploy_both(&builtin_balanced(), &builtin_balanced()).unwrap();
        let score = evaluate(&game.board, Color::Red);
        assert!(score.abs() < 500, "Balanced board should have near-zero score, got {}", score);
    }

    #[test]
    fn test_evaluate_perspective_flip() {
        let mut game = Game::new();
        game.deploy_both(&builtin_balanced(), &builtin_balanced()).unwrap();
        let red_score = evaluate(&game.board, Color::Red);
        let blue_score = evaluate(&game.board, Color::Blue);
        assert_eq!(red_score, -blue_score);
    }
}
