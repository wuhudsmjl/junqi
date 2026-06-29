use junqi_core::board::Board;
use junqi_core::game::Game;
use junqi_core::moves::{execute_move, generate_moves, Move, MoveResult};
use junqi_core::types::Color;

use crate::eval::evaluate;

/// 搜索配置
pub struct SearchConfig {
    /// 最大搜索深度
    pub max_depth: u32,
    /// 是否使用迭代加深
    pub iterative_deepening: bool,
    /// 时间限制（毫秒），0 表示无限制
    pub time_limit_ms: u64,
    /// 随机因子（0.0-1.0），越高越随机
    pub randomness: f64,
}

impl SearchConfig {
    /// 创建指定深度的搜索配置
    pub fn new(depth: u32) -> Self {
        SearchConfig {
            max_depth: depth,
            iterative_deepening: false,
            time_limit_ms: 0,
            randomness: 0.0,
        }
    }
}

/// 搜索最佳走法
///
/// 使用 Minimax + Alpha-Beta 剪枝搜索当前局面的最佳走法。
/// 支持迭代加深：从深度1开始逐步加深至max_depth。
///
/// # 返回
/// 如果存在合法走法，返回最佳走法；否则返回 None。
pub fn find_best_move(game: &Game, config: &SearchConfig) -> Option<Move> {
    let legal_moves = generate_moves(&game.board, game.current_turn);
    if legal_moves.is_empty() {
        return None;
    }

    if config.randomness > 0.0 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < config.randomness {
            let idx = rng.gen_range(0..legal_moves.len());
            return Some(legal_moves[idx]);
        }
    }

    let start_time = std::time::Instant::now();
    let mut best_move: Option<Move> = None;

    let start_depth = if config.iterative_deepening { 1 } else { config.max_depth };
    let mut depth = start_depth;
    loop {
        if depth > config.max_depth {
            break;
        }

        let mut current_best: Option<Move> = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;

        let mut sorted_moves = legal_moves.clone();
        sort_moves(&mut sorted_moves, &game.board);

        for mv in &sorted_moves {
            if config.time_limit_ms > 0
                && start_time.elapsed().as_millis() as u64 > config.time_limit_ms
            {
                break;
            }

            let mut board = game.board.clone();
            let result = execute_move(&mut board, mv);

            if matches!(result, MoveResult::Capture { defender: junqi_core::types::PieceKind::JunQi, .. }) {
                return Some(*mv);
            }

            let score = safe_neg(minimax(
                &board,
                depth - 1,
                safe_neg(beta),
                safe_neg(alpha),
                game.current_turn.opponent(),
            ));

            if score > alpha {
                alpha = score;
                current_best = Some(*mv);
            }
        }

        if let Some(mv) = current_best {
            best_move = Some(mv);
        }

        if !config.iterative_deepening {
            break;
        }

        if config.time_limit_ms > 0
            && start_time.elapsed().as_millis() as u64 > config.time_limit_ms
        {
            break;
        }

        depth += 1;
    }

    best_move.or_else(|| Some(legal_moves[0]))
}

/// Minimax + Alpha-Beta 剪枝
fn minimax(board: &Board, depth: u32, mut alpha: i32, beta: i32, color: Color) -> i32 {
    if depth == 0 {
        return evaluate(board, color);
    }

    let moves = generate_moves(board, color);
    if moves.is_empty() {
        return -100000 + (10 - depth as i32) * 1000;
    }

    let mut sorted_moves = moves.clone();
    if depth >= 2 {
        sort_moves(&mut sorted_moves, board);
    }

    for mv in &sorted_moves {
        let mut new_board = board.clone();
        let result = execute_move(&mut new_board, mv);

        if matches!(result, MoveResult::Capture { defender: junqi_core::types::PieceKind::JunQi, .. }) {
            return 100000;
        }

        let score = safe_neg(minimax(&new_board, depth - 1, safe_neg(beta), safe_neg(alpha), color.opponent()));

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// 走法排序（提高剪枝效率）
///
/// 排序优先级：
/// 1. 吃子走法（MVP — Most Valuable Victim 优先）
/// 2. 移动高价值棋子
fn sort_moves(moves: &mut [Move], board: &Board) {
    moves.sort_by(|a, b| {
        let score_a = move_priority(a, board);
        let score_b = move_priority(b, board);
        score_b.cmp(&score_a)
    });
}

/// 安全取负，避免 i32::MIN 溢出
fn safe_neg(x: i32) -> i32 {
    if x == i32::MIN { i32::MAX } else { -x }
}

/// 走法优先级评分
fn move_priority(mv: &Move, board: &Board) -> i32 {
    let mut score = 0;

    if let Some(target) = board.piece_at(mv.to) {
        score += target.kind.rank() as i32 * 100;
        if let Some(attacker) = board.piece_at(mv.from) {
            if attacker.kind == junqi_core::types::PieceKind::ZhaDan && target.kind.rank() >= 7 {
                score += 500;
            }
        }
    }

    if let Some(piece) = board.piece_at(mv.from) {
        if piece.kind == junqi_core::types::PieceKind::GongBing {
            score += 10;
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use junqi_core::layout::builtin_balanced;

    #[test]
    fn test_find_best_move_returns_something() {
        let mut game = Game::new();
        game.deploy_both(&builtin_balanced(), &builtin_balanced()).unwrap();
        let config = SearchConfig::new(2);
        let best = find_best_move(&game, &config);
        assert!(best.is_some(), "Should find at least one move");
    }

    #[test]
    fn test_search_terminates() {
        let mut game = Game::new();
        game.deploy_both(&builtin_balanced(), &builtin_balanced()).unwrap();
        let config = SearchConfig::new(3);
        let _ = find_best_move(&game, &config);
    }
}
