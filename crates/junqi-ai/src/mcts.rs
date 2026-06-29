use junqi_core::board::Board;
use junqi_core::game::Game;
use junqi_core::moves::{execute_move, generate_moves, Move, MoveResult};
use junqi_core::types::{Color, PieceKind};
use rand::Rng;

use crate::eval::evaluate;

/// MCTS 节点
#[derive(Debug, Clone)]
struct Node {
    /// 走法（从父节点到此节点的走法）
    mv: Option<Move>,
    /// 访问次数
    visits: u32,
    /// 累计得分
    total_score: f64,
    /// 子节点
    children: Vec<Node>,
}

impl Node {
    fn new(mv: Option<Move>) -> Self {
        Node {
            mv,
            visits: 0,
            total_score: 0.0,
            children: Vec::new(),
        }
    }

    /// UCT 值（Upper Confidence Bound for Trees）
    fn uct(&self, parent_visits: u32, exploration: f64) -> f64 {
        if self.visits == 0 {
            return f64::MAX;
        }
        let exploitation = self.total_score / self.visits as f64;
        let exploration_term = exploration * ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        exploitation + exploration_term
    }
}

/// MCTS 搜索配置
pub struct MctsConfig {
    /// 模拟次数
    pub simulations: u32,
    /// 探索常数
    pub exploration: f64,
    /// rollout 深度
    pub rollout_depth: u32,
}

impl Default for MctsConfig {
    fn default() -> Self {
        MctsConfig {
            simulations: 5000,
            exploration: 1.4,
            rollout_depth: 15,
        }
    }
}

/// 使用 MCTS 搜索最佳走法
pub fn mcts_find_best_move(game: &Game, config: &MctsConfig) -> Option<Move> {
    let legal_moves = generate_moves(&game.board, game.current_turn);
    if legal_moves.is_empty() {
        return None;
    }
    if legal_moves.len() == 1 {
        return Some(legal_moves[0]);
    }

    let ai_color = game.current_turn;
    let mut root = Node::new(None);

    for mv in &legal_moves {
        root.children.push(Node::new(Some(*mv)));
    }

    let mut rng = rand::thread_rng();

    for _ in 0..config.simulations {
        let path = select(&root, config.exploration);

        let mut board = deterministic_board(&game.board, ai_color, &mut rng);
        for i in 0..path.len() {
            if let Some(node) = get_node(&root, &path[..=i]) {
                if let Some(mv) = node.mv {
                    execute_move(&mut board, &mv);
                }
            }
        }

        let result = rollout(&board, ai_color, config.rollout_depth, &mut rng);

        backpropagate(&mut root, &path, result);
    }

    root.children
        .iter()
        .max_by(|a, b| a.visits.cmp(&b.visits))
        .and_then(|n| n.mv)
}

/// 根据路径索引获取节点引用
fn get_node<'a>(root: &'a Node, indices: &[usize]) -> Option<&'a Node> {
    let mut current = root;
    for idx in indices {
        current = current.children.get(*idx)?;
    }
    Some(current)
}

/// 选择阶段——从根节点向下选择直到叶节点，返回路径索引
///
/// 使用 UCT 公式选择子节点。遇到未访问过的子节点时停止（需要展开）。
fn select(root: &Node, exploration: f64) -> Vec<usize> {
    let mut path: Vec<usize> = Vec::new();
    let mut current = root;

    loop {
        if current.children.is_empty() {
            break;
        }

        let unvisited_idx = current.children.iter().position(|c| c.visits == 0);
        if let Some(idx) = unvisited_idx {
            path.push(idx);
            break;
        }

        let parent_visits = current.visits;
        let best_idx = current
            .children
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.uct(parent_visits, exploration)
                    .partial_cmp(&b.uct(parent_visits, exploration))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);

        if let Some(idx) = best_idx {
            path.push(idx);
            current = &current.children[idx];
        } else {
            break;
        }
    }

    path
}

/// Determinization：生成确定化棋盘
///
/// 对于暗棋模式下的未知棋子，随机分配给空位和未知位置。
/// 这里简化处理：直接使用当前已知棋盘。
fn deterministic_board(board: &Board, _ai_color: Color, _rng: &mut impl Rng) -> Board {
    board.clone()
}

/// Rollout 阶段——从当前局面随机模拟到一定深度
fn rollout(board: &Board, color: Color, depth: u32, rng: &mut impl Rng) -> f64 {
    let mut current_board = board.clone();
    let mut current_color = color;

    for _ in 0..depth {
        let moves = generate_moves(&current_board, current_color);
        if moves.is_empty() {
            break;
        }

        let idx = rng.gen_range(0..moves.len());
        let mv = moves[idx];
        let result = execute_move(&mut current_board, &mv);

        if matches!(result, MoveResult::Capture { defender: PieceKind::JunQi, .. }) {
            return if current_color == color { 1.0 } else { -1.0 };
        }

        current_color = current_color.opponent();
    }

    let eval = evaluate(&current_board, color);
    (eval as f64 / 20000.0).clamp(-1.0, 1.0)
}

/// 回溯阶段——将模拟结果更新到路径上的所有节点
fn backpropagate(root: &mut Node, path: &[usize], result: f64) {
    root.visits += 1;
    root.total_score += result;
    backpropagate_recursive(root, path, result);
}

fn backpropagate_recursive(node: &mut Node, path: &[usize], result: f64) {
    if let Some((&first, rest)) = path.split_first() {
        if let Some(child) = node.children.get_mut(first) {
            child.visits += 1;
            child.total_score += result;
            backpropagate_recursive(child, rest, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use junqi_core::layout::builtin_balanced;

    #[test]
    fn test_mcts_finds_move() {
        let mut game = Game::new();
        game.deploy_both(&builtin_balanced(), &builtin_balanced()).unwrap();
        let config = MctsConfig {
            simulations: 200,
            exploration: 1.4,
            rollout_depth: 10,
        };
        let best = mcts_find_best_move(&game, &config);
        assert!(best.is_some(), "MCTS should find a move");
    }
}
