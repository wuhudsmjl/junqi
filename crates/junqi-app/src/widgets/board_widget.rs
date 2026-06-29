use std::collections::HashMap;
use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use junqi_core::board::Board;
use junqi_core::moves::Move;
use junqi_core::types::{Color, PieceKind, Position};

pub const FRONTLINE_GAP: f32 = 24.0;

/// 暗棋标注信息：文字和颜色完全独立
///
/// - `text`: 显示在暗棋上的文字（空字符串 = 不显示任何文字）
/// - `color`: 暗棋格子的背景颜色
#[derive(Debug, Clone)]
pub struct AnnotationInfo {
    pub text: String,
    pub color: Color32,
}

impl AnnotationInfo {
    pub fn new(text: String, color: Color32) -> Self {
        AnnotationInfo { text, color }
    }
}

/// 棋盘渲染组件
pub struct BoardWidget {
    pub top_left: Pos2,
    pub cell_size: f32,
    pub selected: Option<Position>,
    pub legal_moves: Vec<Move>,
    pub viewer_color: Color,
    pub reveal_all: bool,
    pub annotations: HashMap<Position, AnnotationInfo>,
    pub flip_board: bool,
    pub last_move_from: Option<Position>,
    pub last_move_to: Option<Position>,
}

impl BoardWidget {
    pub fn new(top_left: Pos2, cell_size: f32) -> Self {
        BoardWidget {
            top_left, cell_size, selected: None, legal_moves: vec![],
            viewer_color: Color::Red, reveal_all: false, annotations: HashMap::new(),
            flip_board: false,
            last_move_from: None, last_move_to: None,
        }
    }
    pub fn total_width(&self) -> f32 { 5.0 * self.cell_size }
    pub fn total_height(&self) -> f32 { 12.0 * self.cell_size + FRONTLINE_GAP }

    fn screen_row(&self, row: u8) -> f32 {
        let r = if self.flip_board { 11 - row } else { row };
        let y_off = if r >= 6 { FRONTLINE_GAP } else { 0.0 };
        self.top_left.y + r as f32 * self.cell_size + y_off
    }

    fn cell_rect(&self, pos: Position) -> Rect {
        let x = self.top_left.x + pos.col as f32 * self.cell_size;
        let y = self.screen_row(pos.row);
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(self.cell_size, self.cell_size))
    }

    fn cell_center(&self, pos: Position) -> Pos2 { self.cell_rect(pos).center() }

    pub fn pos_from_pixel(&self, pixel: Pos2) -> Option<Position> {
        for r in 0..12u8 { for c in 0..5u8 {
            if self.cell_rect(Position::new(r,c)).contains(pixel) { return Some(Position::new(r,c)); }
        }}
        None
    }

    pub fn draw(&mut self, board: &Board, ui: &mut egui::Ui) -> egui::Response {
        let desired = Vec2::new(self.total_width(), self.total_height());
        let (resp, painter) = ui.allocate_painter(desired, egui::Sense::click());
        self.top_left = resp.rect.min;

        for r in 0..12u8 { for c in 0..5u8 {
            let pos = Position::new(r, c);
            let rect = self.cell_rect(pos);
            painter.rect_filled(rect, Rounding::same(0.), cell_bg(board.cell_type(pos), pos));
        }}

        self.draw_roads(&painter);
        self.draw_rails(&painter);
        self.draw_camp_rings(&painter);

        let lbl = egui::FontId::monospace(12.0);
        let mx = self.top_left.x + self.total_width() / 2.0;
        let (top_label, top_color, bot_label, bot_color) = if self.flip_board {
            ("— 蓝方 —", Color32::from_rgb(40, 40, 180),
             "— 红方 —", Color32::from_rgb(180, 40, 40))
        } else {
            ("— 红方 —", Color32::from_rgb(180, 40, 40),
             "— 蓝方 —", Color32::from_rgb(40, 40, 180))
        };
        painter.text(
            Pos2::new(mx, self.top_left.y + 3.0 * self.cell_size),
            egui::Align2::CENTER_CENTER, top_label, lbl.clone(), top_color,
        );
        painter.text(
            Pos2::new(mx, self.top_left.y + 9.0 * self.cell_size + FRONTLINE_GAP),
            egui::Align2::CENTER_CENTER, bot_label, lbl, bot_color,
        );

        let ly = self.top_left.y + 6.0 * self.cell_size + FRONTLINE_GAP / 2.0;
        painter.line_segment(
            [Pos2::new(self.top_left.x, ly), Pos2::new(self.top_left.x + self.total_width(), ly)],
            Stroke::new(2.5, Color32::DARK_GRAY),
        );

        for r in 0..12u8 { for c in 0..5u8 {
            let pos = Position::new(r, c);
            if let Some(piece) = board.piece_at(pos) {
                let visible = self.reveal_all || piece.color == self.viewer_color || piece.revealed;
                let ann = self.annotations.get(&pos);
                self.draw_piece(&painter, pos, piece.color, piece.kind, visible, ann);
            }
        }}

        for r in 0..12u8 { for c in 0..5u8 {
            painter.rect_stroke(
                self.cell_rect(Position::new(r, c)),
                Rounding::same(0.),
                Stroke::new(0.3, Color32::from_gray(180)),
            );
        }}

        resp
    }

    fn draw_camp_rings(&self, painter: &Painter) {
        let camps = [(2u8,1u8),(2,3),(3,2),(4,1),(4,3),(7,1),(7,3),(8,2),(9,1),(9,3)];
        for (r,c) in &camps {
            let ct = self.cell_center(Position::new(*r, *c));
            painter.circle_stroke(ct, self.cell_size * 0.33, Stroke::new(2.0, Color32::from_rgb(80,180,80)));
        }
    }

    fn draw_roads(&self, painter: &Painter) {
        let rs = Stroke::new(0.8, Color32::BLACK);
        let rt = Stroke::new(0.5, Color32::from_gray(80));
        for row in [0u8,2,3,4,7,8,9,11] { for c in 0..4u8 {
            painter.line_segment([self.cell_center(Position::new(row,c)), self.cell_center(Position::new(row,c+1))], rs);
        }}
        for col in 0..5u8 {
            for (r1,r2) in [(0u8,1u8),(10,11)] {
                painter.line_segment([self.cell_center(Position::new(r1,col)), self.cell_center(Position::new(r2,col))], rs);
            }
        }
        for col in [1u8,2,3] { for r in [1u8,2,3,4,6,7,8,9] {
            painter.line_segment([self.cell_center(Position::new(r,col)), self.cell_center(Position::new(r+1,col))], rs);
        }}
        let camps = [(2u8,1u8),(2,3),(3,2),(4,1),(4,3),(7,1),(7,3),(8,2),(9,1),(9,3)];
        for (cr,cc) in &camps {
            let cp = self.cell_center(Position::new(*cr,*cc));
            for (dr,dc) in [(-1i8,-1),(-1,1),(1,-1),(1,1)] {
                let nr = *cr as i8 + dr; let nc = *cc as i8 + dc;
                if nr>=0 && nr<12 && nc>=0 && nc<5 {
                    painter.line_segment([cp, self.cell_center(Position::new(nr as u8, nc as u8))], rt);
                }
            }
        }
    }

    fn draw_rails(&self, painter: &Painter) {
        let mut segments = Vec::new();
        for row in [1u8,5,6,10] { for c in 0..4u8 {
            segments.push((self.cell_center(Position::new(row,c)), self.cell_center(Position::new(row,c+1))));
        }}
        for col in [0u8,4] { for r in 1u8..10u8 {
            segments.push((self.cell_center(Position::new(r,col)), self.cell_center(Position::new(r+1,col))));
        }}
        segments.push((self.cell_center(Position::new(5,2)), self.cell_center(Position::new(6,2))));

        for (a, b) in &segments {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = (dx*dx + dy*dy).sqrt();
            let seg_len = 8.0;
            let n = (len / seg_len) as i32;
            let n = n.max(1);
            for i in 0..n {
                let t0 = i as f32 / n as f32;
                let t1 = (i as f32 + 0.5) / n as f32;
                let p0 = Pos2::new(a.x + dx * t0, a.y + dy * t0);
                let p1 = Pos2::new(a.x + dx * t1, a.y + dy * t1);
                painter.line_segment([p0, p1], Stroke::new(3.5, Color32::BLACK));
                if t1 < 0.99 {
                    let t2 = ((i+1) as f32 / n as f32).min(1.0);
                    let p2 = Pos2::new(a.x + dx * t2, a.y + dy * t2);
                    painter.line_segment([p1, p2], Stroke::new(3.5, Color32::WHITE));
                }
            }
        }
    }

    fn draw_piece(&self, painter: &Painter, pos: Position, color: Color, kind: PieceKind,
                  visible: bool, ann: Option<&AnnotationInfo>) {
        let pw = self.cell_size * 0.72;
        let ph = self.cell_size * 0.44;
        let pr = Rect::from_center_size(self.cell_center(pos), Vec2::new(pw, ph));

        if visible {
            let fill = match color { Color::Red=>Color32::from_rgb(220,100,100), Color::Blue=>Color32::from_rgb(100,140,220) };
            painter.rect_filled(pr, Rounding::same(4.0), fill);
            painter.rect_stroke(pr, Rounding::same(4.0), Stroke::new(1.5, Color32::BLACK));
            let fs = self.cell_size * 0.24;
            painter.text(pr.center(), egui::Align2::CENTER_CENTER, kind.chinese_name(),
                egui::FontId::new(fs, egui::FontFamily::Proportional), Color32::WHITE);
        } else {
            let (bg, txt) = if let Some(a) = ann {
                (a.color, a.text.as_str())
            } else {
                (Color32::from_rgb(140, 140, 140), "")
            };
            painter.rect_filled(pr, Rounding::same(4.0), bg);
            painter.rect_stroke(pr, Rounding::same(4.0), Stroke::new(1.5, Color32::BLACK));
            if !txt.is_empty() {
                let fs = self.cell_size * 0.24;
                let tc = if bg.r() < 80 && bg.g() < 80 && bg.b() < 80 { Color32::WHITE }
                         else { Color32::BLACK };
                painter.text(pr.center(), egui::Align2::CENTER_CENTER, txt,
                    egui::FontId::new(fs, egui::FontFamily::Proportional), tc);
            }
        }
    }

    pub fn draw_overlays(&self, painter: &Painter, board: &Board) {
        if let Some(sel) = self.selected {
            if board.piece_at(sel).is_some() {
                painter.rect_stroke(self.cell_rect(sel), Rounding::same(2.0), Stroke::new(3.0, Color32::YELLOW));
            }
        }
        for mv in &self.legal_moves {
            let c = if board.piece_at(mv.to).is_some() { Color32::from_rgba_premultiplied(255,0,0,120) }
                    else { Color32::from_rgba_premultiplied(0,255,0,120) };
            painter.rect_filled(self.cell_rect(mv.to), Rounding::same(2.0), c);
        }
        if let Some(from) = self.last_move_from {
            painter.rect_stroke(
                self.cell_rect(from), Rounding::same(2.0),
                Stroke::new(3.0, Color32::from_rgb(255, 140, 0)),
            );
        }
        if let Some(to) = self.last_move_to {
            painter.rect_stroke(
                self.cell_rect(to), Rounding::same(2.0),
                Stroke::new(3.0, Color32::from_rgb(0, 180, 80)),
            );
        }
    }
}

fn cell_bg(ct: junqi_core::types::CellType, pos: Position) -> Color32 {
    match ct {
        junqi_core::types::CellType::Station => if (pos.row+pos.col)%2==0 { Color32::from_rgb(245,235,200) } else { Color32::from_rgb(235,225,190) },
        junqi_core::types::CellType::Camp => Color32::from_rgb(170,215,170),
        junqi_core::types::CellType::Headquarters => Color32::from_rgb(215,170,170),
    }
}
