pub mod game_screen;
pub mod layout_editor;
pub mod main_menu;
pub mod replay_viewer;
pub mod room_lobby;
pub mod deploy_screen;
pub mod settings_screen;

use crate::app::JunqiApp;

/// 屏幕路由
pub enum Screen {
    Placeholder,
    MainMenu(main_menu::MainMenuState),
    GameScreen(game_screen::GameScreenState),
    LayoutEditor(layout_editor::LayoutEditorState),
    ReplayViewer(replay_viewer::ReplayViewerState),
    RoomLobby(room_lobby::RoomLobbyState),
    DeployScreen(deploy_screen::DeployScreenState),
    SettingsScreen(settings_screen::SettingsScreenState),
}

impl Screen {
    pub fn main_menu() -> Self { Screen::MainMenu(main_menu::MainMenuState::default()) }

    pub fn show(&mut self, ctx: &egui::Context, app: &mut JunqiApp) {
        egui::CentralPanel::default().show(ctx, |ui| match self {
            Screen::Placeholder => {}
            Screen::MainMenu(s) => s.show(ui, app),
            Screen::GameScreen(s) => s.show(ui, app),
            Screen::LayoutEditor(s) => s.show(ui, app),
            Screen::ReplayViewer(s) => s.show(ui, app),
            Screen::RoomLobby(s) => s.show(ui, app),
            Screen::DeployScreen(s) => s.show(ui, app),
            Screen::SettingsScreen(s) => s.show(ui, app),
        });
    }
}
