use junqi_core::game::Game;
use junqi_core::moves::Move;
use junqi_core::types::AiDifficulty;

use crate::mcts::{mcts_find_best_move, MctsConfig};
use crate::search::{find_best_move, SearchConfig};

/// AI 引擎——根据难度选择最佳走法
///
/// # 难度说明
///
/// | 难度 | 算法 | 搜索深度 | 特点 |
/// |------|------|----------|------|
/// | 初级 | Minimax | 2 | 30% 随机，无位置评估 |
/// | 中级 | Minimax + Alpha-Beta | 4 | 走法排序，基础评估 |
/// | 高级 | Minimax + Alpha-Beta | 6 | 迭代加深，完整评估 |
/// | 专家 | MCTS | 5000 模拟 | 信息不完全博弈优化 |
pub fn ai_find_move(game: &Game, difficulty: AiDifficulty) -> Option<Move> {
    match difficulty {
        AiDifficulty::Easy => {
            let config = SearchConfig {
                max_depth: 2,
                iterative_deepening: false,
                time_limit_ms: 0,
                randomness: 0.30,
            };
            find_best_move(game, &config)
        }
        AiDifficulty::Medium => {
            let config = SearchConfig {
                max_depth: 4,
                iterative_deepening: false,
                time_limit_ms: 0,
                randomness: 0.05,
            };
            find_best_move(game, &config)
        }
        AiDifficulty::Hard => {
            let config = SearchConfig {
                max_depth: 6,
                iterative_deepening: true,
                time_limit_ms: 3000,
                randomness: 0.0,
            };
            find_best_move(game, &config)
        }
        AiDifficulty::Expert => {
            let config = MctsConfig {
                simulations: 5000,
                exploration: 1.4,
                rollout_depth: 20,
            };
            mcts_find_best_move(game, &config)
        }
    }
}
