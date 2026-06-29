/// 军棋 AI 引擎，提供多档难度（初级/中级/高级/专家）的 AI 对战能力。
pub mod difficulty;
/// 局面评估函数，用于对棋盘局势进行打分。
pub mod eval;
/// 蒙特卡洛树搜索 (MCTS) 实现，用于高难度下的信息不完全博弈搜索。
pub mod mcts;
/// Minimax 搜索实现，支持 Alpha-Beta 剪枝与迭代加深。
pub mod search;
