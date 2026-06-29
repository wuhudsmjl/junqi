use junqi_core::game::Game;
use junqi_core::types::Color;

/// 在线对局同步状态
pub struct SyncGameState {
    /// 本地对局
    pub game: Game,
    /// 我方颜色
    pub my_color: Color,
    /// 对方颜色
    pub opponent_color: Color,
    /// 已同步的步数
    pub synced_step: u32,
}

impl SyncGameState {
    /// 创建新的同步对局
    pub fn new(game: Game, my_color: Color) -> Self {
        SyncGameState {
            game,
            my_color,
            opponent_color: my_color.opponent(),
            synced_step: 0,
        }
    }

    /// 是否是轮到我方走棋
    pub fn is_my_turn(&self) -> bool {
        self.game.current_turn == self.my_color
    }
}
