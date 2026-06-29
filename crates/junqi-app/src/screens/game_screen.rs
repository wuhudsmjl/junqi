use egui::{Color32, Pos2, RichText};
use junqi_core::game::{Game, GamePhase, GameResult};
use junqi_core::moves::{Move, MoveResult};
use junqi_core::types::{Color, PieceKind, Position};
use junqi_core::types::AiDifficulty;

use crate::app::JunqiApp;
use crate::screens::main_menu::MainMenuState;
use crate::screens::Screen;
use crate::widgets::board_widget::{AnnotationInfo, BoardWidget};

/// 对局模式
#[derive(Debug, Clone, PartialEq)]
pub enum GameMode { HumanVsAi, #[allow(dead_code)] HotSeat, Online }

/// 一条历史记录
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// 步数编号
    pub step: u32,
    /// 走棋玩家名称
    pub player: String,
    /// 移动起点
    pub from: Position,
    /// 移动终点
    pub to: Position,
    /// 走棋描述
    pub description: String,
}

/// 对局主界面 — 含对局历史面板和暗棋标注
pub struct GameScreenState {
    pub mode: GameMode,
    pub game: Game,
    pub player_color: Color,
    pub ai_difficulty: Option<AiDifficulty>,
    pub player_name: String,
    pub opponent_name: String,
    selected_piece: Option<Position>,
    legal_moves: Vec<Move>,
    game_result: Option<GameResult>,
    history: Vec<HistoryEntry>,
    board_widget: BoardWidget,
    pub online_room: Option<junqi_net::room::OnlineRoom>,
    annotating: Option<Position>,
    show_annot_menu: bool,
    ai_move_pending: bool,
    ai_move_timer: f64,
}

impl GameScreenState {
    pub fn new_human_vs_ai(game: Game, difficulty: AiDifficulty, player_name: String) -> Self {
        GameScreenState {
            mode: GameMode::HumanVsAi, game, player_color: Color::Red,
            ai_difficulty: Some(difficulty), player_name,
            opponent_name: format!("AI({})", difficulty.chinese_name()),
            selected_piece: None, legal_moves: vec![], game_result: None, history: Vec::new(),
            board_widget: BoardWidget::new(Pos2::new(5.0, 5.0), 58.0),
            online_room: None, annotating: None, show_annot_menu: false,
            ai_move_pending: false, ai_move_timer: 0.0,
        }
    }

    /// 如果AI先手（玩家选蓝方），开局立刻让AI走第一步
    pub fn ai_make_first_move(&mut self) {
        if self.game.current_turn != self.player_color {
            self.ai_move();
        }
    }

    pub fn new_online(game: Game, your_color: Color, opponent_name: String, room: junqi_net::room::OnlineRoom) -> Self {
        let player_name = if room.is_host { room.host_name.clone() } else { room.opponent_name.clone().unwrap_or_default() };
        GameScreenState {
            mode: GameMode::Online, game, player_color: your_color,
            ai_difficulty: None, player_name, opponent_name,
            selected_piece: None, legal_moves: vec![], game_result: None, history: Vec::new(),
            board_widget: BoardWidget::new(Pos2::new(5.0, 5.0), 58.0),
            online_room: Some(room), annotating: None, show_annot_menu: false,
            ai_move_pending: false, ai_move_timer: 0.0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        self.board_widget.viewer_color = self.player_color;
        self.board_widget.selected = self.selected_piece;
        self.board_widget.legal_moves = self.legal_moves.clone();
        self.board_widget.flip_board = self.player_color == Color::Red;

        if self.mode == GameMode::Online {
            if self.game.phase == GamePhase::Playing && self.game.current_turn != self.player_color {
                ui.ctx().request_repaint();
            }
            if let Some(ref room) = self.online_room {
                while let Some(msg) = room.client.try_recv() {
                    match msg {
                        junqi_net::protocol::ServerMessage::GameMove { mv, step_number: _ } => {
                            let from = mv.from;
                            let to = mv.to;
                            match self.game.make_move(&mv) {
                                Ok(result) => {
                                    let desc = self.fmt_result(&result);
                                    self.history.push(HistoryEntry {
                                        step: self.game.step_count,
                                        player: self.opponent_name.clone(),
                                        from, to,
                                        description: desc,
                                    });
                                    self.board_widget.last_move_from = Some(from);
                                    self.board_widget.last_move_to = Some(to);
                                }
                                Err(junqi_core::game::GameError::GameOver(result)) => {
                                    self.history.push(HistoryEntry {
                                        step: self.game.step_count,
                                        player: self.opponent_name.clone(),
                                        from, to,
                                        description: "对局结束!".to_string(),
                                    });
                                    self.board_widget.last_move_from = Some(from);
                                    self.board_widget.last_move_to = Some(to);
                                    self.game_result = Some(result);
                                }
                                Err(_) => {}
                            }
                        }
                        junqi_net::protocol::ServerMessage::Surrender => {
                            self.game_result = Some(self.game.surrender());
                        }
                        junqi_net::protocol::ServerMessage::ChatMessage { from, text } => {
                            self.history.push(HistoryEntry {
                                step: self.game.step_count,
                                player: from,
                                from: Position::new(0, 0),
                                to: Position::new(0, 0),
                                description: format!("💬 {}", text),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.ai_move_pending && self.game.phase == GamePhase::Playing {
            let dt = ui.ctx().input(|i| i.unstable_dt) as f64;
            self.ai_move_timer += dt;
            if self.ai_move_timer >= 1.0 {
                self.ai_move_pending = false;
                self.ai_move_timer = 0.0;
                self.ai_move();
            }
            ui.ctx().request_repaint();
        }

        egui::SidePanel::right("history_panel").min_width(250.0).show_inside(ui, |ui| {
            self.history_panel(ui);
        });
        egui::SidePanel::left("info_panel").min_width(150.0).show_inside(ui, |ui| {
            self.info_panel(ui, app);
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                let resp = self.board_widget.draw(&self.game.board, ui);
                let right_click = resp.clicked_by(egui::PointerButton::Secondary)
                    || ui.ctx().input(|i| i.pointer.secondary_clicked() && resp.hovered());
                if right_click {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        self.handle_annotate_click(pos);
                    }
                } else if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        self.handle_move_click(pos);
                    }
                }
                let painter = ui.painter_at(resp.rect);
                self.board_widget.draw_overlays(&painter, &self.game.board);
            });
        });

        self.show_annotation_popup(ui);
    }

    fn handle_move_click(&mut self, click_pos: Pos2) {
        if self.game.phase != GamePhase::Playing { return; }
        if self.mode == GameMode::Online && self.game.current_turn != self.player_color { return; }

        if let Some(pos) = self.board_widget.pos_from_pixel(click_pos) {
            if let Some(piece) = self.game.board.piece_at(pos) {
                if piece.color == self.game.current_turn {
                    self.selected_piece = Some(pos);
                    let p = piece.clone();
                    self.legal_moves = junqi_core::moves::generate_piece_moves(&self.game.board, pos, &p);
                    return;
                }
            }
            if let Some(from) = self.selected_piece {
                let mv = Move::new(from, pos);
                if self.legal_moves.iter().any(|m| m.to == pos) {
                    self.do_move(mv);
                    return;
                }
            }
            self.selected_piece = None;
            self.legal_moves.clear();
        }
    }

    fn handle_annotate_click(&mut self, click_pos: Pos2) {
        if let Some(pos) = self.board_widget.pos_from_pixel(click_pos) {
            if let Some(piece) = self.game.board.piece_at(pos) {
                if piece.color != self.player_color && !piece.revealed {
                    self.annotating = Some(pos);
                    self.show_annot_menu = true;
                }
            }
        }
    }

    fn show_annotation_popup(&mut self, ui: &mut egui::Ui) {
        if !self.show_annot_menu { return; }

        let colors: [(Color32, &str); 8] = [
            (Color32::RED, "红"),
            (Color32::from_rgb(255, 140, 0), "橙"),
            (Color32::from_rgb(255, 200, 0), "黄"),
            (Color32::GREEN, "绿"),
            (Color32::from_rgb(0, 180, 220), "青"),
            (Color32::BLUE, "蓝"),
            (Color32::from_rgb(180, 0, 200), "紫"),
            (Color32::from_rgb(140, 140, 140), "灰"),
        ];

        let all_kinds: [PieceKind; 12] = [
            PieceKind::SiLing, PieceKind::JunZhang, PieceKind::ShiZhang,
            PieceKind::LvZhang, PieceKind::TuanZhang, PieceKind::YingZhang,
            PieceKind::LianZhang, PieceKind::PaiZhang, PieceKind::GongBing,
            PieceKind::ZhaDan, PieceKind::DiLei, PieceKind::JunQi,
        ];

        if let Some(pos) = self.annotating {
            let mut close_popup = false;

            egui::Window::new("标注暗棋")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false).resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("标注位置: ({}, {})", pos.row, pos.col));
                    ui.separator();

                    ui.label(RichText::new("文字标注:").strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui.add_sized([44.0, 28.0], egui::Button::new("!!!")).clicked() {
                            self.ann_set_text(pos, "!!!".to_string());
                            close_popup = true;
                        }
                        if ui.add_sized([44.0, 28.0], egui::Button::new("???")).clicked() {
                            self.ann_set_text(pos, "???".to_string());
                            close_popup = true;
                        }
                        ui.separator();
                        if ui.button("清除文字").clicked() {
                            self.ann_set_text(pos, String::new());
                            close_popup = true;
                        }
                    });

                    ui.add_space(4.0);

                    egui::Grid::new("piece_grid").num_columns(4).show(ui, |ui| {
                        for k in &all_kinds {
                            if ui.add_sized([52.0, 24.0], egui::Button::new(k.chinese_name())).clicked() {
                                self.ann_set_text(pos, k.chinese_name().to_string());
                                close_popup = true;
                            }
                            ui.end_row();
                        }
                    });

                    ui.separator();

                    ui.label(RichText::new("颜色标注:").strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        for (c, name) in &colors {
                            if ui.add(egui::Button::new("■").fill(*c).min_size(egui::vec2(28.0, 22.0)))
                                .on_hover_text(*name).clicked()
                            {
                                self.ann_set_color(pos, *c);
                                close_popup = true;
                            }
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("清除标注").clicked() {
                            self.board_widget.annotations.remove(&pos);
                            close_popup = true;
                        }
                        if ui.button("关闭").clicked() {
                            close_popup = true;
                        }
                    });
                });

            if close_popup {
                self.show_annot_menu = false;
            }
        }
    }

    fn ann_set_text(&mut self, pos: Position, text: String) {
        let color = self.board_widget.annotations.get(&pos)
            .map(|a| a.color)
            .unwrap_or(Color32::from_rgb(140, 140, 140));
        self.board_widget.annotations.insert(pos, AnnotationInfo::new(text, color));
    }

    fn ann_set_color(&mut self, pos: Position, color: Color32) {
        let text = self.board_widget.annotations.get(&pos)
            .map(|a| a.text.clone())
            .unwrap_or_default();
        self.board_widget.annotations.insert(pos, AnnotationInfo::new(text, color));
    }

    fn do_move(&mut self, mv: Move) {
        let from_str = format!("({},{})", mv.from.row, mv.from.col);
        let to_str = format!("({},{})", mv.to.row, mv.to.col);

        match self.game.make_move(&mv) {
            Ok(result) => {
                let desc = self.fmt_result(&result);
                self.history.push(HistoryEntry { step: self.game.step_count, player: self.player_name.clone(), from: mv.from, to: mv.to, description: desc.clone() });
                self.board_widget.last_move_from = Some(mv.from);
                self.board_widget.last_move_to = Some(mv.to);
                self.selected_piece = None;
                self.legal_moves.clear();

                if self.mode == GameMode::Online {
                    if let Some(ref room) = self.online_room {
                        let _ = room.send_move(&mv, self.game.step_count);
                    }
                } else if self.mode == GameMode::HumanVsAi && self.game.phase == GamePhase::Playing {
                    self.ai_move_pending = true;
                    self.ai_move_timer = 0.0;
                }
            }
            Err(junqi_core::game::GameError::GameOver(result)) => {
                self.board_widget.last_move_from = Some(mv.from);
                self.board_widget.last_move_to = Some(mv.to);
                self.history.push(HistoryEntry { step: self.game.step_count, player: self.player_name.clone(), from: mv.from, to: mv.to,
                    description: format!("{}→{} 对局结束!", from_str, to_str) });
                self.game_result = Some(result);
                self.save_replay();
            }
            Err(e) => {
                self.history.push(HistoryEntry { step: self.game.step_count, player: self.player_name.clone(), from: mv.from, to: mv.to,
                    description: format!("错误: {}", e) });
            }
        }
    }

    fn ai_move(&mut self) {
        let d = self.ai_difficulty.unwrap_or(AiDifficulty::Medium);
        if let Some(best) = junqi_ai::difficulty::ai_find_move(&self.game, d) {
            match self.game.make_move(&best) {
                Ok(result) => {
                    let desc = self.fmt_result(&result);
                    self.history.push(HistoryEntry { step: self.game.step_count, player: self.opponent_name.clone(), from: best.from, to: best.to, description: desc });
                    self.board_widget.last_move_from = Some(best.from);
                    self.board_widget.last_move_to = Some(best.to);
                }
                Err(junqi_core::game::GameError::GameOver(result)) => {
                    self.board_widget.last_move_from = Some(best.from);
                    self.board_widget.last_move_to = Some(best.to);
                    self.history.push(HistoryEntry { step: self.game.step_count, player: self.opponent_name.clone(), from: best.from, to: best.to,
                        description: "对局结束!".to_string() });
                    self.game_result = Some(result);
                    self.save_replay();
                }
                Err(_) => {}
            }
        }
    }

    fn fmt_result(&self, r: &MoveResult) -> String {
        match r {
            MoveResult::Moved => "移动".into(),
            MoveResult::Capture { attacker: _, defender: _ } => "吃子成功".into(),
            MoveResult::Defeated { attacker: _, defender: _ } => "攻击失败，被反吃!".into(),
            MoveResult::MutualDestruction { attacker: _, defender: _ } => "同归于尽!".into(),
            MoveResult::Invalid => "无效".into(),
        }
    }

    fn save_replay(&self) {
        let r = self.game.to_replay(&self.player_name, &self.opponent_name);
        let _ = junqi_storage::replay_store::save_replay(&r);
    }

    fn info_panel(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.heading("对局信息");
        let (my_label, opp_label) = if self.player_color == Color::Red {
            ("红方(你)", "蓝方")
        } else {
            ("红方", "蓝方(你)")
        };
        ui.label(RichText::new(format!("{}: {}", my_label, self.player_name)).color(Color32::from_rgb(200,50,50)));
        ui.label(RichText::new(format!("{}: {}", opp_label, self.opponent_name)).color(Color32::from_rgb(50,50,200)));
        let turn_text = if self.game.current_turn == self.player_color {
            format!("轮到：你 ({})", self.game.current_turn.chinese_name())
        } else {
            format!("等待：对方 ({})", self.game.current_turn.chinese_name())
        };
        let turn_color = if self.game.current_turn == Color::Red { Color32::from_rgb(200,50,50) }
                         else { Color32::from_rgb(50,50,200) };
        ui.label(RichText::new(turn_text).color(turn_color));
        ui.label(format!("步数: {}", self.game.step_count));
        ui.label(RichText::new("右键点击对方暗棋 → 标注").size(11.0).color(Color32::GRAY));
        ui.separator();
        if self.game.phase == GamePhase::Playing {
            if ui.button("认输").clicked() {
                self.game_result = Some(self.game.surrender());
                if self.mode == GameMode::Online {
                    if let Some(ref room) = self.online_room {
                        let _ = room.surrender();
                    }
                }
                self.save_replay();
            }
        }
        if let Some(ref r) = self.game_result {
            egui::Window::new("结果").anchor(egui::Align2::CENTER_CENTER,[0.,0.]).collapsible(false).resizable(false).show(ui.ctx(),|ui|{
                ui.heading(match r{GameResult::RedWins{..}=>"红方胜!",GameResult::BlueWins{..}=>"蓝方胜!",GameResult::Draw{..}=>"平局!"});
                if ui.button("返回").clicked(){app.navigate_to(Screen::MainMenu(MainMenuState::default()));}
            });
        }
        ui.add_space(10.0);
        if ui.button("← 返回").clicked() { app.navigate_to(Screen::MainMenu(MainMenuState::default())); }
    }

    fn history_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("对局历史");
        let h = self.history.clone();
        egui::ScrollArea::vertical().max_height(300.0).stick_to_bottom(true).show(ui, |ui| {
            for entry in &h {
                ui.label(RichText::new(format!("#{} {}: ({},{})→({},{}) | {}",
                    entry.step, entry.player, entry.from.row, entry.from.col,
                    entry.to.row, entry.to.col, entry.description)).size(12.0));
            }
        });
    }
}
