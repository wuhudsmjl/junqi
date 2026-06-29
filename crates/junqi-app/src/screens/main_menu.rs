use egui::{Align2, Color32, RichText, Vec2};
use junqi_core::types::AiDifficulty;

use crate::app::JunqiApp;
use crate::screens::deploy_screen::DeployScreenState;
use crate::screens::layout_editor::LayoutEditorState;
use crate::screens::replay_viewer::ReplayViewerState;
use crate::screens::room_lobby::RoomLobbyState;
use crate::screens::settings_screen::SettingsScreenState;
use crate::screens::Screen;

/// 主菜单界面
#[derive(Default)]
pub struct MainMenuState {
    show_difficulty_popup: bool,
    selected_difficulty: AiDifficulty,
}

impl MainMenuState {
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("军 棋").size(48.0).color(Color32::from_rgb(180, 30, 30)));
            ui.label(RichText::new("陆 战 棋").size(24.0).color(Color32::DARK_GRAY));
            ui.add_space(30.0);

            let btn_size = Vec2::new(260.0, 50.0);

            if ui.add_sized(btn_size, egui::Button::new(RichText::new("🤖  人机对战").size(20.0))).clicked() {
                self.show_difficulty_popup = true;
                self.selected_difficulty = app.settings.ai_difficulty;
            }

            if self.show_difficulty_popup {
                egui::Window::new("选择 AI 难度")
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.set_min_width(200.0);
                        for diff in &AiDifficulty::all() {
                            let sel = self.selected_difficulty == *diff;
                            if ui.selectable_label(sel, diff.chinese_name()).clicked() {
                                self.selected_difficulty = *diff;
                            }
                        }
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("取消").clicked() {
                                self.show_difficulty_popup = false;
                            }
                            if ui.button("开始对战").clicked() {
                                self.show_difficulty_popup = false;
                                app.navigate_to(Screen::DeployScreen(DeployScreenState::new(
                                    app.settings.player_name.clone(),
                                    self.selected_difficulty,
                                )));
                            }
                        });
                    });
            }

            ui.add_space(10.0);
            if ui.add_sized(btn_size, egui::Button::new(RichText::new("🌐  网络对战").size(20.0))).clicked() {
                app.navigate_to(Screen::RoomLobby(RoomLobbyState::default()));
            }
            ui.add_space(10.0);
            if ui.add_sized(btn_size, egui::Button::new(RichText::new("✏ 布阵管理").size(20.0))).clicked() {
                app.navigate_to(Screen::LayoutEditor(LayoutEditorState::new()));
            }
            ui.add_space(10.0);
            if ui.add_sized(btn_size, egui::Button::new(RichText::new("📖  复盘查看").size(20.0))).clicked() {
                app.navigate_to(Screen::ReplayViewer(ReplayViewerState::new()));
            }
            ui.add_space(10.0);
            if ui.add_sized(btn_size, egui::Button::new(RichText::new("⚙  设置").size(20.0))).clicked() {
                app.navigate_to(Screen::SettingsScreen(SettingsScreenState::new(app)));
            }
            ui.add_space(20.0);
            ui.label(RichText::new(format!("玩家：{}", app.settings.player_name)).size(14.0).color(Color32::GRAY));
        });
    }
}
