use egui::{Color32, Pos2, RichText};
use junqi_core::board::Board;
use junqi_core::game::Game;
use junqi_core::moves::execute_move;
use junqi_core::replay::Replay;
use junqi_core::types::{Color, AiDifficulty};
use junqi_storage::replay_store;

use crate::app::JunqiApp;
use crate::screens::game_screen::GameScreenState;
use crate::screens::main_menu::MainMenuState;
use crate::screens::Screen;
use crate::widgets::board_widget::BoardWidget;

/// 复盘查看器
///
/// 与人机对战相同的三栏布局。每步 = 单方走棋。
/// 可从任意残局状态继续对战（专家 AI）。
pub struct ReplayViewerState {
    replays: Vec<replay_store::ReplayInfo>,
    selected_idx: Option<usize>,
    current_step: usize,
    total_steps: usize,
    board: Board,
    viewer_color: Color,
    current_replay: Option<Replay>,
    cached_replay: Option<Replay>,
    current_step_desc: String,
    replay_result_text: String,
    board_widget: BoardWidget,
}

impl ReplayViewerState {
    pub fn new() -> Self {
        let mut state = ReplayViewerState {
            replays: Vec::new(),
            selected_idx: None,
            current_step: 0,
            total_steps: 0,
            board: Board::new(),
            viewer_color: Color::Red,
            current_replay: None,
            cached_replay: None,
            current_step_desc: String::new(),
            replay_result_text: String::new(),
            board_widget: BoardWidget::new(Pos2::new(5.0, 5.0), 55.0),
        };
        state.refresh_list();
        state
    }

    fn refresh_list(&mut self) {
        self.replays = replay_store::load_replays().unwrap_or_default();
    }

    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        if self.replays.is_empty() {
            self.refresh_list();
        }

        egui::SidePanel::right("replay_right")
            .min_width(220.0)
            .show_inside(ui, |ui| {
                self.right_panel(ui, app);
            });

        egui::SidePanel::left("replay_left")
            .min_width(260.0)
            .show_inside(ui, |ui| {
                self.left_panel(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                self.board_widget.reveal_all = true;
                self.board_widget.viewer_color = self.viewer_color;
                self.board_widget.flip_board = self.viewer_color == Color::Red;
                self.board_widget.draw(&self.board, ui);
            });
        });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("复盘列表");
        if ui.button("刷新").clicked() {
            self.refresh_list();
        }
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut clicked_idx = None;
            for (i, info) in self.replays.iter().enumerate() {
                let selected = self.selected_idx == Some(i);
                let saved_mark = if info.replay.saved { "💾" } else { "  " };
                let label = format!(
                    "{}{} vs {}\n  {} 步",
                    saved_mark,
                    info.replay.red_name,
                    info.replay.blue_name,
                    info.replay.total_steps()
                );
                if ui.selectable_label(selected, label).clicked() {
                    clicked_idx = Some(i);
                }
                ui.separator();
            }
            if let Some(i) = clicked_idx {
                self.select_replay(i);
            }
        });
    }

    fn right_panel(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        if self.current_replay.is_none() {
            ui.label(RichText::new("← 请选择一份复盘").color(Color32::GRAY));
        } else {
            ui.heading("复盘控制");

            let (filename, is_saved) = self.selected_info()
                .map(|(f, r)| (f, r.saved))
                .unwrap_or_default();
            let status = if filename.is_empty() { "" } else if is_saved { "💾 已保存" } else { "⚠ 未保存" };
            if !status.is_empty() {
                ui.label(RichText::new(status).size(13.0).color(if is_saved { Color32::GREEN } else { Color32::from_rgb(200, 150, 0) }));
            }

            if !self.replay_result_text.is_empty() {
                ui.label(RichText::new(&self.replay_result_text).color(Color32::from_rgb(200, 150, 0)));
            }

            ui.horizontal(|ui| {
                if filename.is_empty() { return; }
                if is_saved {
                    if ui.button("取消保存").clicked() {
                        let _ = replay_store::set_saved(&filename, false);
                        self.refresh_after_action();
                    }
                } else {
                    if ui.button("💾 保存").clicked() {
                        let _ = replay_store::set_saved(&filename, true);
                        self.refresh_after_action();
                    }
                }
                if ui.button(RichText::new("🗑 删除").color(Color32::RED)).clicked() {
                    let _ = replay_store::delete_replay(&filename);
                    self.selected_idx = None;
                    self.current_replay = None;
                    self.cached_replay = None;
                    self.refresh_list();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.selectable_label(self.viewer_color == Color::Red, "红方视角").clicked() {
                    self.viewer_color = Color::Red;
                    self.rebuild_board();
                }
                if ui.selectable_label(self.viewer_color == Color::Blue, "蓝方视角").clicked() {
                    self.viewer_color = Color::Blue;
                    self.rebuild_board();
                }
            });

            ui.separator();

            ui.label(format!("步数: {} / {}", self.current_step, self.total_steps));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("⏮").on_hover_text("最开始").clicked() {
                    self.goto_step(0);
                }
                if ui.button("◀").on_hover_text("上一步").clicked() && self.current_step > 0 {
                    self.goto_step(self.current_step - 1);
                }
                if ui.button("▶").on_hover_text("下一步").clicked() && self.current_step < self.total_steps {
                    self.goto_step(self.current_step + 1);
                }
                if ui.button("⏭").on_hover_text("最后").clicked() {
                    self.goto_step(self.total_steps);
                }
            });

            ui.add_space(8.0);

            let mut step = self.current_step;
            let slider = egui::Slider::new(&mut step, 0..=self.total_steps).text("跳转");
            if ui.add(slider).changed() {
                self.goto_step(step);
            }

            ui.separator();

            ui.label(RichText::new(&self.current_step_desc).size(13.0));

            ui.separator();

            if ui.button(RichText::new("⚔ 从此残局继续对战").size(14.0).color(Color32::GREEN)).clicked() {
                self.continue_from_position(app);
            }

            ui.add_space(8.0);
        }

        ui.add_space(16.0);
        if ui.button("← 返回主菜单").clicked() {
            app.navigate_to(Screen::MainMenu(MainMenuState::default()));
        }
    }

    fn selected_info(&self) -> Option<(String, &Replay)> {
        let idx = self.selected_idx?;
        let info = self.replays.get(idx)?;
        Some((info.filename.clone(), &info.replay))
    }

    fn refresh_after_action(&mut self) {
        if let Some(idx) = self.selected_idx {
            let filename = self.replays.get(idx).map(|r| r.filename.clone());
            self.refresh_list();
            if let Some(fname) = filename {
                if let Some(new_idx) = self.replays.iter().position(|r| r.filename == fname) {
                    self.selected_idx = Some(new_idx);
                    if let Some(info) = self.replays.get(new_idx) {
                        self.current_replay = Some(info.replay.clone());
                    }
                }
            }
        }
    }

    fn select_replay(&mut self, idx: usize) {
        if let Some(info) = self.replays.get(idx) {
            self.selected_idx = Some(idx);
            let replay = info.replay.clone();
            self.replay_result_text = replay.result_description();
            self.total_steps = replay.total_steps();
            self.viewer_color = Color::Red;
            self.cached_replay = Some(replay.clone());
            self.current_replay = Some(replay);
            self.goto_step(0);
        }
    }

    fn goto_step(&mut self, step: usize) {
        self.current_step = step.min(self.total_steps);
        self.update_step_desc();
        self.rebuild_board();
    }

    fn update_step_desc(&mut self) {
        if self.current_step == 0 {
            self.current_step_desc = "初始布阵".to_string();
        } else if let Some(ref replay) = self.current_replay {
            if let Some(step) = replay.step(self.current_step - 1) {
                let who = match step.turn {
                    Color::Red => "红方",
                    Color::Blue => "蓝方",
                };
                self.current_step_desc = format!(
                    "第{}步 [{}]: ({},{})→({},{}) | {}",
                    self.current_step,
                    who,
                    step.mv.from.row, step.mv.from.col,
                    step.mv.to.row, step.mv.to.col,
                    move_result_desc(&step.result)
                );
            }
        }
    }

    fn rebuild_board(&mut self) {
        let replay = match &self.cached_replay { Some(r) => r, None => return };
        let red_layout = match replay.red_layout { Some(ref l) => l.clone(), None => return };
        let blue_layout = match replay.blue_layout { Some(ref l) => l.clone(), None => return };

        let mut board = Board::new();
        red_layout.apply_to_board(&mut board, Color::Red);
        blue_layout.apply_to_board(&mut board, Color::Blue);

        for (i, step) in replay.steps.iter().enumerate() {
            if i >= self.current_step { break; }
            execute_move(&mut board, &step.mv);
        }

        self.board = board;
    }

    fn continue_from_position(&mut self, app: &mut JunqiApp) {
        let replay = match &self.cached_replay { Some(r) => r.clone(), None => return };

        match Game::from_replay(&replay, Some(self.current_step)) {
            Ok(game) => {
                if game.phase == junqi_core::game::GamePhase::Finished {
                    log::warn!("无法从已结束的对局继续");
                    return;
                }
                let turn = game.current_turn;
                let mut state = GameScreenState::new_human_vs_ai(
                    game,
                    AiDifficulty::Expert,
                    format!("{}（续）", app.settings.player_name),
                );
                state.player_color = self.viewer_color;
                if turn != self.viewer_color {
                    state.ai_make_first_move();
                }
                app.navigate_to(Screen::GameScreen(state));
            }
            Err(e) => {
                log::error!("重建对局失败: {:?}", e);
            }
        }
    }
}

fn move_result_desc(result: &junqi_core::moves::MoveResult) -> String {
    match result {
        junqi_core::moves::MoveResult::Moved => "移动".to_string(),
        junqi_core::moves::MoveResult::Capture { attacker, defender } =>
            format!("{}吃{}", attacker.chinese_name(), defender.chinese_name()),
        junqi_core::moves::MoveResult::Defeated { attacker, defender } =>
            format!("{}攻{}败", attacker.chinese_name(), defender.chinese_name()),
        junqi_core::moves::MoveResult::MutualDestruction { attacker, defender } =>
            format!("{}与{}同归于尽", attacker.chinese_name(), defender.chinese_name()),
        junqi_core::moves::MoveResult::Invalid => "无效".to_string(),
    }
}
