use std::collections::HashMap;
use egui::{Align2, Color32, Pos2, RichText};
use junqi_core::board::Board;
use junqi_core::layout::{Layout, builtin_layouts};
use junqi_core::piece::Piece;
use junqi_core::types::{Color, PieceKind, Position};
use junqi_storage::layout_store;
use rand::Rng;

use crate::app::JunqiApp;
use crate::screens::game_screen::GameScreenState;
use crate::screens::main_menu::MainMenuState;
use crate::screens::Screen;
use crate::widgets::board_widget::BoardWidget;

const ALL_KINDS: [PieceKind; 12] = [
    PieceKind::SiLing, PieceKind::JunZhang, PieceKind::ShiZhang,
    PieceKind::LvZhang, PieceKind::TuanZhang, PieceKind::YingZhang,
    PieceKind::LianZhang, PieceKind::PaiZhang, PieceKind::GongBing,
    PieceKind::ZhaDan, PieceKind::DiLei, PieceKind::JunQi,
];

fn validate_placement(kind: PieceKind, pos: Position) -> Result<(), String> {
    if junqi_core::board::Board::is_camp_position(pos) {
        return Err("行营不可布子".into());
    }
    if kind == PieceKind::JunQi {
        let is_hq = pos.row == 0 && (pos.col == 1 || pos.col == 3);
        if !is_hq { return Err("军旗必须放在大本营".into()); }
    }
    if kind == PieceKind::DiLei && pos.row != 0 && pos.row != 1 {
        return Err("地雷只能放在末两排".into());
    }
    if kind == PieceKind::ZhaDan && pos.row == 5 {
        return Err("炸弹不能放在前线".into());
    }
    Ok(())
}

/// 战前布阵界面
///
/// 使用与对局界面相同的 BoardWidget 渲染棋盘，
/// 玩家在己方领土上布置棋子。
pub struct DeployScreenState {
    board_pieces: HashMap<Position, PieceKind>,
    pool: Vec<(PieceKind, u8)>,
    selected_pool: Option<PieceKind>,
    show_layout_picker: bool,
    save_name: String,
    ai_difficulty: junqi_core::types::AiDifficulty,
    player_name: String,
    player_side: Color,
    #[allow(dead_code)]
    is_human_vs_human: bool,
    message: Option<String>,
    board_widget: BoardWidget,
}

impl DeployScreenState {
    pub fn new(player_name: String, difficulty: junqi_core::types::AiDifficulty) -> Self {
        let mut s = DeployScreenState {
            board_pieces: HashMap::new(),
            pool: Vec::new(),
            selected_pool: None,
            show_layout_picker: false,
            save_name: String::new(),
            ai_difficulty: difficulty,
            player_name,
            player_side: Color::Red,
            is_human_vs_human: false,
            message: None,
            board_widget: BoardWidget::new(Pos2::new(5.0, 5.0), 55.0),
        };
        let def = builtin_layouts().get(2).cloned().unwrap_or_else(|| builtin_layouts()[0].clone());
        s.load_layout(&def);
        s.board_widget.reveal_all = true;
        s
    }

    fn load_layout(&mut self, layout: &Layout) {
        self.board_pieces.clear();
        for &(kind, pos) in &layout.pieces {
            self.board_pieces.insert(pos, kind);
        }
        self.update_pool();
        self.selected_pool = None;
    }

    fn update_pool(&mut self) {
        let mut counts: HashMap<PieceKind, u8> = HashMap::new();
        for k in &ALL_KINDS {
            counts.insert(*k, k.count_per_side());
        }
        for (_, kind) in &self.board_pieces {
            *counts.get_mut(kind).unwrap() -= 1;
        }
        self.pool = ALL_KINDS.iter().filter_map(|k| {
            let c = *counts.get(k).unwrap();
            if c > 0 { Some((*k, c)) } else { None }
        }).collect();
    }

    fn build_temp_board(&self) -> Board {
        let mut board = Board::new();
        for (&red_pos, &kind) in &self.board_pieces {
            let actual = self.to_actual_pos(red_pos);
            board.place_piece(actual, Piece::new(kind, self.player_side));
        }
        board
    }

    fn to_actual_pos(&self, red_pos: Position) -> Position {
        match self.player_side {
            Color::Red => red_pos,
            Color::Blue => Position::new(11 - red_pos.row, red_pos.col),
        }
    }

    fn from_actual_pos(&self, actual: Position) -> Option<Position> {
        match self.player_side {
            Color::Red if actual.row <= 5 => Some(actual),
            Color::Blue if actual.row >= 6 => Some(Position::new(11 - actual.row, actual.col)),
            _ => None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        let temp_board = self.build_temp_board();
        self.board_widget.flip_board = self.player_side == Color::Red;

        egui::SidePanel::right("deploy_right")
            .min_width(220.0)
            .show_inside(ui, |ui| {
                self.right_panel(ui, app);
            });

        egui::SidePanel::left("deploy_left")
            .min_width(160.0)
            .show_inside(ui, |ui| {
                self.left_panel(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                let resp = self.board_widget.draw(&temp_board, ui);

                if resp.clicked() {
                    if let Some(click_pos) = resp.interact_pointer_pos() {
                        self.handle_board_click(click_pos);
                    }
                }
            });
        });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("布阵");
        ui.label("你的阵地");
        ui.add_space(4.0);

        if let Some(ref m) = self.message {
            ui.label(RichText::new(m).color(Color32::YELLOW));
            ui.add_space(4.0);
        }

        ui.separator();

        ui.label("阵营:");
        ui.horizontal(|ui| {
            if ui.selectable_label(self.player_side == Color::Red, "🔴 红方(先手)").clicked() {
                self.player_side = Color::Red;
            }
            if ui.selectable_label(self.player_side == Color::Blue, "🔵 蓝方(后手)").clicked() {
                self.player_side = Color::Blue;
            }
        });

        ui.separator();

        if let Some(sel) = self.selected_pool {
            ui.label(
                RichText::new(format!("已选中: {}", sel.chinese_name()))
                    .color(Color32::GREEN),
            );
            ui.label("点击己方领土空位放置");
        } else {
            ui.label("选择棋子后点击空位放置");
        }
        ui.label("点击已有棋子 → 取下");
        ui.add_space(6.0);

        self.draw_pool(ui);
    }

    fn right_panel(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.heading("操作");

        ui.add_space(4.0);

        if ui.button("📋 布阵模板").clicked() {
            self.show_layout_picker = true;
        }

        if self.show_layout_picker {
            egui::Window::new("选择布阵")
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    let all = layout_store::all_layouts().unwrap_or_else(|_| builtin_layouts());
                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        for l in &all {
                            if ui.button(&l.name).clicked() {
                                self.load_layout(l);
                                self.show_layout_picker = false;
                            }
                        }
                    });
                    if ui.button("取消").clicked() {
                        self.show_layout_picker = false;
                    }
                });
        }

        ui.add_space(4.0);

        if ui.button("🎲 随机打乱").clicked() {
            self.random_shuffle();
        }
        if ui.button("🪞 镜像翻转").clicked() {
            self.mirror_flip();
        }

        ui.separator();

        ui.label("保存名称:");
        ui.text_edit_singleline(&mut self.save_name);
        if ui.button("💾 保存布阵").clicked() {
            self.save(app);
        }

        ui.separator();

        let start_btn = egui::Button::new(
            RichText::new("⚔ 开始对战!")
                .size(16.0)
                .color(Color32::GREEN),
        );
        if ui.add_sized([160.0, 36.0], start_btn).clicked() {
            self.start_game(app);
        }

        ui.add_space(8.0);

        if ui.button("← 返回").clicked() {
            app.navigate_to(Screen::MainMenu(MainMenuState::default()));
        }
    }

    fn draw_pool(&mut self, ui: &mut egui::Ui) {
        let pool = self.pool.clone();
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("pool_grid").num_columns(3).show(ui, |ui| {
                    for (kind, count) in &pool {
                        let txt = format!("{}×{}", kind.chinese_name(), count);
                        let sel = self.selected_pool == Some(*kind);
                        let btn = if sel {
                            egui::Button::new(
                                RichText::new(&txt)
                                    .size(13.0)
                                    .color(Color32::GREEN)
                                    .strong(),
                            )
                        } else {
                            egui::Button::new(RichText::new(&txt).size(13.0))
                        };
                        if ui.add_sized([70.0, 26.0], btn).clicked() {
                            self.selected_pool = Some(*kind);
                        }
                        ui.end_row();
                    }
                });
            });
    }

    fn handle_board_click(&mut self, click_pos: Pos2) {
        if let Some(actual) = self.board_widget.pos_from_pixel(click_pos) {
            if let Some(red_pos) = self.from_actual_pos(actual) {
                if junqi_core::board::Board::is_camp_position(red_pos) {
                    self.message = Some("行营不可布子".to_string());
                    return;
                }
                let is_hq = red_pos.row == 0 && (red_pos.col == 1 || red_pos.col == 3);

                if self.board_pieces.contains_key(&red_pos) {
                    self.board_pieces.remove(&red_pos);
                    self.update_pool();
                    self.message = None;
                } else if let Some(kind) = self.selected_pool {
                    if is_hq {
                        if kind != PieceKind::JunQi {
                            let other_hq = if red_pos.col == 1 {
                                Position::new(0, 3)
                            } else {
                                Position::new(0, 1)
                            };
                            if self.board_pieces.get(&other_hq) != Some(&PieceKind::JunQi) {
                                self.message = Some("大本营需要先放军旗".to_string());
                                return;
                            }
                        }
                    }
                    match validate_placement(kind, red_pos) {
                        Ok(()) => {
                            self.board_pieces.insert(red_pos, kind);
                            self.update_pool();
                            self.selected_pool = None;
                            self.message = None;
                        }
                        Err(e) => {
                            self.message = Some(format!("无法放置: {}", e));
                        }
                    }
                }
            }
        }
    }

    fn random_shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut assignment = HashMap::new();

        let all_positions: Vec<Position> = (0u8..6)
            .flat_map(|r| (0u8..5).map(move |c| Position::new(r, c)))
            .filter(|p| !junqi_core::board::Board::is_camp_position(*p))
            .collect();

        let hq = if rng.gen_bool(0.5) {
            Position::new(0, 1)
        } else {
            Position::new(0, 3)
        };
        assignment.insert(hq, PieceKind::JunQi);

        let mine_pool: Vec<Position> = [0u8, 1]
            .iter()
            .flat_map(|r| (0u8..5).map(move |c| Position::new(*r, c)))
            .filter(|p| !junqi_core::board::Board::is_camp_position(*p) && *p != hq)
            .collect();
        let mut mine_pool = mine_pool;
        mine_pool.shuffle(&mut rng);
        for i in 0..3 {
            assignment.insert(mine_pool[i], PieceKind::DiLei);
        }

        let bomb_pool: Vec<Position> = all_positions
            .iter()
            .filter(|p| !assignment.contains_key(p) && p.row != 5)
            .cloned()
            .collect();
        let mut bomb_pool = bomb_pool;
        bomb_pool.shuffle(&mut rng);
        for i in 0..2 {
            assignment.insert(bomb_pool[i], PieceKind::ZhaDan);
        }

        let remaining_pieces: Vec<PieceKind> = ALL_KINDS
            .iter()
            .filter(|k| {
                **k != PieceKind::JunQi
                    && **k != PieceKind::DiLei
                    && **k != PieceKind::ZhaDan
            })
            .flat_map(|k| vec![*k; k.count_per_side() as usize])
            .collect();
        let mut remaining_pieces = remaining_pieces;
        remaining_pieces.shuffle(&mut rng);

        let remaining_positions: Vec<Position> = all_positions
            .iter()
            .filter(|p| !assignment.contains_key(p))
            .cloned()
            .collect();

        let pos_count = remaining_positions.len();
        let piece_count = remaining_pieces.len();
        if pos_count != piece_count {
            log::error!(
                "random_shuffle: positions {} != pieces {}",
                pos_count,
                piece_count
            );
            self.message = Some("随机布阵生成失败，请重试".to_string());
            return;
        }
        for (pos, kind) in remaining_positions.iter().zip(remaining_pieces.iter()) {
            assignment.insert(*pos, *kind);
        }

        self.board_pieces = assignment;
        self.update_pool();
        self.selected_pool = None;
        self.message = None;
    }

    fn mirror_flip(&mut self) {
        let old = self.board_pieces.clone();
        self.board_pieces.clear();
        for (&pos, &kind) in &old {
            self.board_pieces
                .insert(Position::new(pos.row, 4 - pos.col), kind);
        }
        self.selected_pool = None;
    }

    fn save(&mut self, app: &mut JunqiApp) {
        if self.save_name.trim().is_empty() {
            self.message = Some("请先输入保存名称".to_string());
            return;
        }
        let mut l = Layout::new_for_color(self.save_name.clone(), "自定义".to_string(), self.player_side);
        for (&p, &k) in &self.board_pieces {
            l.add_piece(k, p);
        }
        match l.validate(Color::Red) {
            Ok(()) => match layout_store::save_layout(&l) {
                Ok(()) => {
                    app.refresh_layouts();
                    self.message = Some(format!("布阵「{}」已保存", self.save_name));
                }
                Err(e) => {
                    self.message = Some(format!("保存失败: {}", e));
                }
            },
            Err(e) => {
                self.message = Some(format!("布阵无效: {}", e));
            }
        }
    }

    fn start_game(&mut self, app: &mut JunqiApp) {
        if !self.save_name.is_empty() {
            self.save(app);
        }

        let mut l = Layout::new("custom", "");
        for (&p, &k) in &self.board_pieces {
            l.add_piece(k, p);
        }
        match l.validate(self.player_side) {
            Ok(()) => {
                let mut game = junqi_core::game::Game::new();
                let ai_l = builtin_layouts()
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| builtin_layouts()[0].clone());
                match self.player_side {
                    Color::Red => {
                        game.deploy_red(&l).ok();
                        game.deploy_blue(&ai_l).ok();
                    }
                    Color::Blue => {
                        game.deploy_red(&ai_l).ok();
                        game.deploy_blue(&l).ok();
                    }
                }
                if self.player_side == Color::Blue {
                    game.current_turn = Color::Red;
                }
                let mut state = GameScreenState::new_human_vs_ai(
                    game,
                    self.ai_difficulty,
                    self.player_name.clone(),
                );
                state.player_color = self.player_side;
                if self.player_side == Color::Blue {
                    state.ai_make_first_move();
                }
                app.navigate_to(Screen::GameScreen(state));
            }
            Err(e) => {
                self.message = Some(format!("布阵无效: {}", e));
            }
        }
    }
}
