use std::collections::HashMap;
use egui::{Align2, Color32, Pos2, RichText};
use junqi_core::board::Board;
use junqi_core::layout::{Layout, builtin_layouts};
use junqi_core::piece::Piece;
use junqi_core::types::{Color, PieceKind, Position};
use junqi_storage::layout_store;
use rand::Rng;

use crate::app::JunqiApp;
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

/// 布阵管理
///
/// 与人机对战布阵阶段使用相同的 BoardWidget。
/// 三栏排布：左（备选池）| 中（棋盘）| 右（操作）。
/// "加载已有布阵"在编辑器内弹窗，不跳转页面。
pub struct LayoutEditorState {
    board_pieces: HashMap<Position, PieceKind>,
    pool: Vec<(PieceKind, u8)>,
    selected_pool: Option<PieceKind>,
    layout_name: String,
    description: String,
    edit_color: Color,
    loaded_from_name: Option<String>,
    message: Option<String>,
    show_layout_picker: bool,
    picker_filter_color: Color,
    board_widget: BoardWidget,
}

impl LayoutEditorState {
    pub fn new() -> Self {
        let mut s = LayoutEditorState {
            board_pieces: HashMap::new(),
            pool: Vec::new(),
            selected_pool: None,
            layout_name: String::new(),
            description: String::new(),
            edit_color: Color::Red,
            loaded_from_name: None,
            message: None,
            show_layout_picker: false,
            picker_filter_color: Color::Red,
            board_widget: BoardWidget::new(Pos2::new(5.0, 5.0), 55.0),
        };
        s.board_widget.reveal_all = true;
        s.reset_pool();
        let def = builtin_layouts().get(2).cloned().unwrap_or_else(|| builtin_layouts()[0].clone());
        s.load_layout(&def);
        s
    }

    fn reset_pool(&mut self) {
        self.pool = ALL_KINDS.iter().map(|k| (*k, k.count_per_side())).collect();
    }

    fn update_pool(&mut self) {
        let mut counts: HashMap<PieceKind, u8> = HashMap::new();
        for k in &ALL_KINDS { counts.insert(*k, k.count_per_side()); }
        for (_, kind) in &self.board_pieces { *counts.get_mut(kind).unwrap() -= 1; }
        self.pool = ALL_KINDS.iter().filter_map(|k| {
            let c = *counts.get(k).unwrap();
            if c > 0 { Some((*k, c)) } else { None }
        }).collect();
    }

    fn load_layout(&mut self, layout: &Layout) {
        self.board_pieces.clear();
        for &(kind, pos) in &layout.pieces {
            self.board_pieces.insert(pos, kind);
        }
        self.layout_name = layout.name.clone();
        self.description = layout.description.clone();
        if let Some(c) = layout.color { self.edit_color = c; }
        self.loaded_from_name = Some(layout.name.clone());
        self.update_pool();
        self.selected_pool = None;
        self.message = None;
    }

    fn build_temp_board(&self) -> Board {
        let mut board = Board::new();
        for (&red_pos, &kind) in &self.board_pieces {
            let actual = match self.edit_color {
                Color::Red => red_pos,
                Color::Blue => Position::new(11 - red_pos.row, red_pos.col),
            };
            board.place_piece(actual, Piece::new(kind, self.edit_color));
        }
        board
    }

    fn from_actual_pos(&self, actual: Position) -> Option<Position> {
        match self.edit_color {
            Color::Red if actual.row <= 5 => Some(actual),
            Color::Blue if actual.row >= 6 => Some(Position::new(11 - actual.row, actual.col)),
            _ => None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        let temp_board = self.build_temp_board();
        self.board_widget.flip_board = self.edit_color == Color::Red;

        egui::SidePanel::right("editor_right")
            .min_width(200.0)
            .show_inside(ui, |ui| {
                self.right_panel(ui, app);
            });

        egui::SidePanel::left("editor_left")
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

        self.show_layout_picker_popup(ui, app);
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("编辑布阵");

        ui.label("编辑阵营:");
        ui.horizontal(|ui| {
            if ui.selectable_label(self.edit_color == Color::Red, "🔴 红方").clicked() {
                self.edit_color = Color::Red;
            }
            if ui.selectable_label(self.edit_color == Color::Blue, "🔵 蓝方").clicked() {
                self.edit_color = Color::Blue;
            }
        });

        ui.separator();

        if let Some(ref m) = self.message {
            ui.label(RichText::new(m).color(Color32::YELLOW));
            ui.add_space(4.0);
        }

        if let Some(sel) = self.selected_pool {
            ui.label(RichText::new(format!("已选中: {}", sel.chinese_name())).color(Color32::GREEN));
            ui.label("点击己方领土空位放置");
        } else {
            ui.label("选择棋子后点击空位放置");
        }
        ui.label("点击已有棋子 → 取下");
        ui.add_space(6.0);

        let pool = self.pool.clone();
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            egui::Grid::new("pool_grid").num_columns(3).show(ui, |ui| {
                for (kind, count) in &pool {
                    let txt = format!("{}×{}", kind.chinese_name(), count);
                    let sel = self.selected_pool == Some(*kind);
                    let btn = if sel {
                        egui::Button::new(RichText::new(&txt).size(13.0).color(Color32::GREEN).strong())
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

    fn right_panel(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.heading("布阵信息");

        ui.label("名称:");
        ui.text_edit_singleline(&mut self.layout_name);
        ui.label("描述:");
        ui.text_edit_singleline(&mut self.description);

        ui.separator();

        if ui.button("📋 加载已有布阵").clicked() {
            app.refresh_layouts();
            self.picker_filter_color = self.edit_color;
            self.show_layout_picker = true;
        }

        if ui.button("🎲 随机打乱").clicked() {
            self.random_shuffle();
        }
        if ui.button("🪞 镜像翻转").clicked() {
            self.mirror_flip();
        }
        if ui.button("🗑 清空棋盘").clicked() {
            self.board_pieces.clear();
            self.reset_pool();
            self.selected_pool = None;
            self.loaded_from_name = None;
            self.layout_name.clear();
            self.description.clear();
            self.message = None;
        }

        ui.separator();

        if let Some(ref orig_name) = self.loaded_from_name.clone() {
            ui.label(RichText::new(format!("来源: {}", orig_name)).size(12.0).color(Color32::GRAY));
            ui.horizontal(|ui| {
                if ui.button("💾 覆盖原布阵").clicked() {
                    self.overwrite(app, orig_name);
                }
                if ui.button("📄 另存为新布阵").clicked() {
                    self.save_as_new(app);
                }
            });
            if ui.button(RichText::new("🗑 删除此布阵").color(Color32::RED)).clicked() {
                self.delete_loaded(app, orig_name);
            }
        } else {
            if ui.button("💾 保存布阵").clicked() {
                self.save_as_new(app);
            }
        }

        ui.add_space(16.0);
        if ui.button("← 返回主菜单").clicked() {
            app.navigate_to(Screen::MainMenu(MainMenuState::default()));
        }
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
                    if is_hq && kind != PieceKind::JunQi {
                        let other = if red_pos.col == 1 { Position::new(0, 3) } else { Position::new(0, 1) };
                        if self.board_pieces.get(&other) != Some(&PieceKind::JunQi) {
                            self.message = Some("大本营需要先放军旗".to_string());
                            return;
                        }
                    }
                    match validate_placement(kind, red_pos) {
                        Ok(()) => {
                            self.board_pieces.insert(red_pos, kind);
                            self.update_pool();
                            self.selected_pool = None;
                            self.message = None;
                        }
                        Err(e) => { self.message = Some(format!("无法放置: {}", e)); }
                    }
                }
            }
        }
    }

    fn show_layout_picker_popup(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        if !self.show_layout_picker { return; }

        egui::Window::new("选择布阵")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false).resizable(false)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);

                ui.horizontal(|ui| {
                    ui.label("显示:");
                    if ui.selectable_label(self.picker_filter_color == Color::Red, "红方").clicked() {
                        self.picker_filter_color = Color::Red;
                    }
                    if ui.selectable_label(self.picker_filter_color == Color::Blue, "蓝方").clicked() {
                        self.picker_filter_color = Color::Blue;
                    }
                });
                ui.separator();

                let all = app.available_layouts.clone();
                let filtered: Vec<&Layout> = all.iter().filter(|l| {
                    l.color.is_none() || l.color == Some(self.picker_filter_color)
                }).collect();

                if filtered.is_empty() {
                    ui.label("没有匹配的布阵");
                }

                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    for layout in &filtered {
                        let color_tag = match layout.color {
                            Some(Color::Red) => "🔴",
                            Some(Color::Blue) => "🔵",
                            None => "⬜",
                        };
                        let resp = ui.selectable_label(
                            false,
                            RichText::new(format!("{} {} — {}", color_tag, layout.name, layout.description)).size(14.0),
                        );
                        if resp.clicked() {
                            self.load_layout(layout);
                            self.show_layout_picker = false;
                        }
                        resp.on_hover_ui(|ui| {
                            let mut counts: HashMap<PieceKind, u8> = HashMap::new();
                            for &(k, _) in &layout.pieces { *counts.entry(k).or_insert(0) += 1; }
                            let mut kinds: Vec<&PieceKind> = counts.keys().collect();
                            kinds.sort_by_key(|k| std::cmp::Reverse(k.rank()));
                            for k in kinds {
                                ui.label(format!("{}×{}", k.chinese_name(), counts[k]));
                            }
                        });
                    }
                });

                ui.separator();
                if ui.button("关闭").clicked() {
                    self.show_layout_picker = false;
                }
            });
    }

    fn overwrite(&mut self, app: &mut JunqiApp, orig_name: &str) {
        let mut l = Layout::new_for_color(orig_name.to_string(), self.description.clone(), self.edit_color);
        for (&p, &k) in &self.board_pieces { l.add_piece(k, p); }
        match l.validate(Color::Red) {
            Ok(()) => {
                if self.layout_name.trim() != orig_name {
                    let _ = layout_store::delete_layout(orig_name);
                }
                match layout_store::save_layout(&l) {
                    Ok(()) => {
                        app.refresh_layouts();
                        self.layout_name = orig_name.to_string();
                        self.loaded_from_name = Some(orig_name.to_string());
                        self.message = Some(format!("已覆盖「{}」", orig_name));
                    }
                    Err(e) => { self.message = Some(format!("保存失败: {}", e)); }
                }
            }
            Err(e) => { self.message = Some(format!("布阵无效: {}", e)); }
        }
    }

    fn save_as_new(&mut self, app: &mut JunqiApp) {
        let name = self.layout_name.trim();
        if name.is_empty() {
            self.message = Some("请先输入布阵名称".to_string());
            return;
        }
        let mut l = Layout::new_for_color(name.to_string(), self.description.clone(), self.edit_color);
        for (&p, &k) in &self.board_pieces { l.add_piece(k, p); }
        match l.validate(Color::Red) {
            Ok(()) => match layout_store::save_layout(&l) {
                Ok(()) => {
                    app.refresh_layouts();
                    self.loaded_from_name = Some(name.to_string());
                    self.message = Some(format!("布阵「{}」已保存", name));
                }
                Err(e) => { self.message = Some(format!("保存失败: {}", e)); }
            },
            Err(e) => { self.message = Some(format!("布阵无效: {}", e)); }
        }
    }

    fn delete_loaded(&mut self, app: &mut JunqiApp, name: &str) {
        match layout_store::delete_layout(name) {
            Ok(()) => {
                app.refresh_layouts();
                self.board_pieces.clear();
                self.reset_pool();
                self.selected_pool = None;
                self.layout_name.clear();
                self.description.clear();
                self.loaded_from_name = None;
                self.message = Some(format!("已删除「{}」", name));
            }
            Err(e) => { self.message = Some(format!("删除失败: {}", e)); }
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

        let hq = if rng.gen_bool(0.5) { Position::new(0, 1) } else { Position::new(0, 3) };
        assignment.insert(hq, PieceKind::JunQi);

        let mine_pool: Vec<Position> = [0u8, 1].iter()
            .flat_map(|r| (0u8..5).map(move |c| Position::new(*r, c)))
            .filter(|p| !junqi_core::board::Board::is_camp_position(*p) && *p != hq)
            .collect();
        let mut mine_pool = mine_pool; mine_pool.shuffle(&mut rng);
        for i in 0..3 { assignment.insert(mine_pool[i], PieceKind::DiLei); }

        let bomb_pool: Vec<Position> = all_positions.iter()
            .filter(|p| !assignment.contains_key(p) && p.row != 5).cloned().collect();
        let mut bomb_pool = bomb_pool; bomb_pool.shuffle(&mut rng);
        for i in 0..2 { assignment.insert(bomb_pool[i], PieceKind::ZhaDan); }

        let remaining_pieces: Vec<PieceKind> = ALL_KINDS.iter()
            .filter(|k| **k != PieceKind::JunQi && **k != PieceKind::DiLei && **k != PieceKind::ZhaDan)
            .flat_map(|k| vec![*k; k.count_per_side() as usize]).collect();
        let mut remaining_pieces = remaining_pieces; remaining_pieces.shuffle(&mut rng);

        let remaining_positions: Vec<Position> = all_positions.iter()
            .filter(|p| !assignment.contains_key(p)).cloned().collect();

        if remaining_positions.len() != remaining_pieces.len() {
            self.message = Some("随机布阵生成失败".to_string());
            return;
        }
        for (pos, kind) in remaining_positions.iter().zip(remaining_pieces.iter()) {
            assignment.insert(*pos, *kind);
        }

        self.board_pieces = assignment;
        self.update_pool();
        self.selected_pool = None;
        self.loaded_from_name = None;
        self.layout_name.clear();
        self.description.clear();
        self.message = None;
    }

    fn mirror_flip(&mut self) {
        let old = self.board_pieces.clone();
        self.board_pieces.clear();
        for (&pos, &kind) in &old {
            self.board_pieces.insert(Position::new(pos.row, 4 - pos.col), kind);
        }
        self.selected_pool = None;
    }
}
