use egui::RichText;
use junqi_core::types::AiDifficulty;
use junqi_storage::settings::save_settings;

use crate::app::JunqiApp;
use crate::screens::main_menu::MainMenuState;
use crate::screens::Screen;

/// 设置界面
pub struct SettingsScreenState {
    player_name: String,
    ai_difficulty: AiDifficulty,
    sound_enabled: bool,
    message: Option<String>,
}

impl Default for SettingsScreenState {
    fn default() -> Self {
        SettingsScreenState {
            player_name: String::new(),
            ai_difficulty: AiDifficulty::Medium,
            sound_enabled: true,
            message: None,
        }
    }
}

impl SettingsScreenState {
    /// 使用应用当前设置创建设置界面状态
    pub fn new(app: &JunqiApp) -> Self {
        SettingsScreenState {
            player_name: app.settings.player_name.clone(),
            ai_difficulty: app.settings.ai_difficulty,
            sound_enabled: app.settings.sound_enabled,
            message: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.heading("设置");
        ui.separator();

        ui.label("玩家名称:");
        ui.text_edit_singleline(&mut self.player_name);

        ui.add_space(8.0);

        ui.label("AI 默认难度:");
        for diff in &AiDifficulty::all() {
            let sel = self.ai_difficulty == *diff;
            if ui.selectable_label(sel, diff.chinese_name()).clicked() {
                self.ai_difficulty = *diff;
            }
        }

        ui.add_space(8.0);

        ui.checkbox(&mut self.sound_enabled, "启用音效");

        ui.add_space(16.0);
        ui.separator();

        if ui.button("💾 保存设置").clicked() {
            app.settings.player_name = self.player_name.clone();
            app.settings.ai_difficulty = self.ai_difficulty;
            app.settings.sound_enabled = self.sound_enabled;
            match save_settings(&app.settings) {
                Ok(()) => {
                    self.message = Some("设置已保存".to_string());
                }
                Err(e) => {
                    self.message = Some(format!("保存失败: {}", e));
                }
            }
        }

        if let Some(ref msg) = self.message {
            ui.add_space(4.0);
            ui.label(RichText::new(msg).color(egui::Color32::GREEN));
        }

        ui.add_space(16.0);
        if ui.button("← 返回主菜单").clicked() {
            app.navigate_to(Screen::MainMenu(MainMenuState::default()));
        }
    }
}
