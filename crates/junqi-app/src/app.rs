use eframe::egui;
use junqi_core::layout::{builtin_layouts, Layout};
use junqi_storage::layout_store;
use junqi_storage::settings::{load_settings, Settings};

use crate::screens::Screen;

/// 应用全局状态与屏幕路由
pub struct JunqiApp {
    pub current_screen: Screen,
    pub settings: Settings,
    pub available_layouts: Vec<Layout>,
}

impl Default for JunqiApp {
    fn default() -> Self {
        let settings = load_settings();
        let available_layouts = layout_store::all_layouts().unwrap_or_else(|_| builtin_layouts());
        JunqiApp {
            current_screen: Screen::main_menu(),
            settings,
            available_layouts,
        }
    }
}

impl eframe::App for JunqiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.data(|d| d.get_temp::<bool>(egui::Id::new("fonts_loaded")).unwrap_or(false)) {
            self.setup_fonts(ctx);
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("fonts_loaded"), true));
        }
        let mut screen = std::mem::replace(&mut self.current_screen, Screen::Placeholder);
        screen.show(ctx, self);
        if matches!(self.current_screen, Screen::Placeholder) {
            self.current_screen = screen;
        }
    }
}

impl JunqiApp {
    fn setup_fonts(&self, ctx: &egui::Context) {
        use egui::{FontDefinitions, FontFamily, FontData};
        let mut fonts = FontDefinitions::default();
        let chinese_fonts = [
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "/System/Library/Fonts/PingFang.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ];
        let mut loaded = false;
        for path in &chinese_fonts {
            if std::path::Path::new(path).exists() {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert("chinese".to_string(), FontData::from_owned(data).into());
                    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "chinese".to_string());
                    fonts.families.entry(FontFamily::Monospace).or_default().push("chinese".to_string());
                    loaded = true;
                    break;
                }
            }
        }
        if !loaded {
            log::warn!("未找到中文字体");
        }
        ctx.set_fonts(fonts);
    }

    /// 导航到指定屏幕
    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    /// 从存储刷新可用布阵列表
    pub fn refresh_layouts(&mut self) {
        self.available_layouts = layout_store::all_layouts().unwrap_or_else(|_| builtin_layouts());
    }
}
