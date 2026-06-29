use egui::{Color32, RichText};
use junqi_core::game::Game;
use junqi_core::layout::builtin_layouts;
use junqi_net::protocol::ServerMessage;
use junqi_net::room::{OnlineRoom, RoomState};

use crate::app::JunqiApp;
use crate::screens::game_screen::GameScreenState;
use crate::screens::main_menu::MainMenuState;
use crate::screens::Screen;

/// 房间大厅界面
///
/// 创建/加入房间、等待对手、开始在线对局。
pub struct RoomLobbyState {
    server_addr: String,
    room: Option<OnlineRoom>,
    room_code_input: String,
    player_name: String,
    chat_input: String,
    chat_log: Vec<String>,
    error_msg: Option<String>,
    game_started: bool,
}

impl Default for RoomLobbyState {
    fn default() -> Self {
        RoomLobbyState {
            server_addr: "127.0.0.1:9876".to_string(),
            room: None,
            room_code_input: String::new(),
            player_name: "玩家".to_string(),
            chat_input: String::new(),
            chat_log: Vec::new(),
            error_msg: None,
            game_started: false,
        }
    }
}

impl RoomLobbyState {
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        self.player_name = app.settings.player_name.clone();

        if let Some(ref mut room) = self.room {
            let msgs = room.poll();
            self.handle_messages(msgs, app);
        }

        if self.game_started {
            return;
        }

        ui.heading("网络对战");
        ui.separator();

        if self.room.is_some() {
            self.draw_room_view(ui, app);
        } else {
            self.draw_lobby_view(ui, app);
        }

        if let Some(ref msg) = self.error_msg {
            ui.add_space(5.0);
            ui.label(RichText::new(msg).color(Color32::RED));
        }

        ui.add_space(16.0);
        if ui.button("← 返回主菜单").clicked() {
            self.cleanup();
            app.navigate_to(Screen::MainMenu(MainMenuState::default()));
        }
    }

    fn draw_lobby_view(&mut self, ui: &mut egui::Ui, app: &mut JunqiApp) {
        ui.label("服务器地址:");
        ui.text_edit_singleline(&mut self.server_addr);
        ui.label("玩家名称:");
        ui.text_edit_singleline(&mut self.player_name);
        app.settings.player_name = self.player_name.clone();

        ui.add_space(12.0);
        ui.separator();
        ui.heading("创建房间");
        if ui.add_sized([200.0, 36.0], egui::Button::new("🏠 创建房间")).clicked() {
            self.create_room();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("加入房间");
        ui.label("输入房间号:");
        ui.text_edit_singleline(&mut self.room_code_input);
        if ui.add_sized([200.0, 36.0], egui::Button::new("🔍 加入")).clicked() {
            self.join_room();
        }
    }

    fn draw_room_view(&mut self, ui: &mut egui::Ui, _app: &mut JunqiApp) {
        let room = match &self.room {
            Some(r) => r,
            None => return,
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("房间号: {}", room.room_code)).size(28.0).strong());
            ui.add_space(16.0);
            if room.is_host {
                ui.label(RichText::new("房主").color(Color32::GREEN));
            } else {
                ui.label(RichText::new("访客").color(Color32::YELLOW));
            }
        });

        ui.separator();

        ui.label(format!("房主: {}", room.host_name));
        if let Some(ref opp) = room.opponent_name {
            ui.label(format!("对手: {}", opp));
        } else {
            ui.label(RichText::new("等待对手加入...").color(Color32::GRAY));
        }

        ui.separator();

        if room.is_host && room.opponent_name.is_some() && room.state == RoomState::Waiting {
            if ui.add_sized([200.0, 40.0],
                egui::Button::new(RichText::new("▶ 开始对局").size(16.0).color(Color32::GREEN))
            ).clicked() {
                let _ = room.start_game();
            }
        } else if !room.is_host && room.state == RoomState::Waiting {
            ui.label(RichText::new("等待房主开始对局...").color(Color32::GRAY));
        }

        ui.add_space(12.0);
        ui.separator();
        ui.label("聊天:");
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            for msg in &self.chat_log {
                ui.label(RichText::new(msg).size(12.0));
            }
        });
        ui.horizontal(|ui| {
            let resp = ui.text_edit_singleline(&mut self.chat_input);
            let send = ui.button("发送").clicked()
                || (resp.lost_focus() && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)));
            if send && !self.chat_input.is_empty() {
                if let Some(ref room) = self.room {
                    let _ = room.send_chat(&self.chat_input);
                    self.chat_log.push(format!("我: {}", self.chat_input));
                    self.chat_input.clear();
                }
            }
        });
    }

    fn handle_messages(&mut self, msgs: Vec<ServerMessage>, app: &mut JunqiApp) {
        for msg in msgs {
            match msg {
                ServerMessage::ChatMessage { from, text } => {
                    self.chat_log.push(format!("{}: {}", from, text));
                }
                ServerMessage::GameStarted { opponent_name, your_color } => {
                    if self.game_started { return; }
                    self.game_started = true;

                    let layout = builtin_layouts()
                        .get(2).cloned()
                        .unwrap_or_else(|| builtin_layouts()[0].clone());
                    let mut game = Game::new();
                    let _ = game.deploy_both(&layout, &layout);

                    let room = self.room.take().expect("room must exist");
                    let state = GameScreenState::new_online(
                        game, your_color, opponent_name, room,
                    );
                    app.navigate_to(Screen::GameScreen(state));
                }
                ServerMessage::PlayerLeft { player_name } => {
                    self.chat_log.push(format!("[系统] {} 离开了", player_name));
                    if let Some(ref room) = self.room {
                        if Some(&player_name) == room.opponent_name.as_ref() {
                            self.error_msg = Some("对手已离开".to_string());
                        }
                    }
                }
                ServerMessage::Kicked { reason } => {
                    self.error_msg = Some(format!("被踢出: {}", reason));
                    self.cleanup();
                }
                ServerMessage::RoomDisbanded => {
                    self.error_msg = Some("房间已解散".to_string());
                    self.cleanup();
                }
                _ => {}
            }
        }
    }

    fn create_room(&mut self) {
        let addr = self.server_addr.parse().unwrap_or_else(|_| "127.0.0.1:9876".parse().unwrap());
        match OnlineRoom::create(&self.player_name, addr) {
            Ok(room) => {
                self.chat_log.push(format!("[系统] 房间创建成功! 房间号: {}", room.room_code));
                self.room = Some(room);
                self.error_msg = None;
            }
            Err(e) => {
                self.error_msg = Some(format!("创建失败: {}", e));
            }
        }
    }

    fn join_room(&mut self) {
        let code: u16 = match self.room_code_input.trim().parse() {
            Ok(c) => c,
            Err(_) => {
                self.error_msg = Some("请输入有效的房间号".to_string());
                return;
            }
        };
        let addr = self.server_addr.parse().unwrap_or_else(|_| "127.0.0.1:9876".parse().unwrap());
        match OnlineRoom::join(code, &self.player_name, addr) {
            Ok(room) => {
                self.chat_log.push(format!("[系统] 已加入房间 {}", code));
                self.room = Some(room);
                self.error_msg = None;
            }
            Err(e) => {
                self.error_msg = Some(format!("加入失败: {}", e));
            }
        }
    }

    fn cleanup(&mut self) {
        self.room = None;
        self.game_started = false;
    }
}
