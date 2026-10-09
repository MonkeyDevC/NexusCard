//! Visual building blocks: gradient background, glass panels, gradient buttons,
//! drop zone, segmented control and the saved-card row.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2};

pub const BG_TOP: Color32 = Color32::from_rgb(15, 16, 32);
pub const BG_BOTTOM: Color32 = Color32::from_rgb(6, 7, 12);
pub const ACCENT_A: Color32 = Color32::from_rgb(98, 64, 220);
pub const ACCENT_B: Color32 = Color32::from_rgb(142, 100, 255);
pub const TEXT: Color32 = Color32::from_rgb(236, 234, 244);
pub const TEXT_DIM: Color32 = Color32::from_rgb(168, 166, 184);
pub const OK: Color32 = Color32::from_rgb(88, 214, 141);

pub fn glass_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 12)
}

pub fn glass_stroke() -> Stroke {
    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28))
}

fn gradient_rect(painter: &egui::Painter, rect: Rect, tl: Color32, tr: Color32, bl: Color32, br: Color32) {
    let mut mesh = egui::Mesh::default();
    let idx = mesh.vertices.len() as u32;
    for (pos, color) in [
        (rect.left_top(), tl),
        (rect.right_top(), tr),
        (rect.right_bottom(), br),
        (rect.left_bottom(), bl),
    ] {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: egui::epaint::WHITE_UV,
            color,
        });
    }
    mesh.indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
    painter.add(Shape::mesh(mesh));
}

fn glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: egui::epaint::WHITE_UV,
        color,
    });
    let steps = 40;
    for i in 0..=steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + Vec2::angled(angle) * radius,
            uv: egui::epaint::WHITE_UV,
            color: Color32::TRANSPARENT,
        });
        if i > 0 {
            mesh.indices.extend_from_slice(&[0, i as u32, i as u32 + 1]);
        }
    }
    painter.add(Shape::mesh(mesh));
}

/// Paints the window background behind all panels.
pub fn paint_background(ctx: &egui::Context) {
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    gradient_rect(&painter, rect, BG_TOP, BG_TOP, BG_BOTTOM, BG_BOTTOM);
    glow(
        &painter,
        rect.left_top() + Vec2::new(rect.width() * 0.18, 20.0),
        rect.width() * 0.45,
        Color32::from_rgba_unmultiplied(110, 70, 230, 70),
    );
    glow(
        &painter,
        rect.right_top() + Vec2::new(-rect.width() * 0.1, 40.0),
        rect.width() * 0.35,
        Color32::from_rgba_unmultiplied(60, 110, 230, 40),
    );
}

pub fn glass_panel<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(glass_fill())
        .stroke(glass_stroke())
        .corner_radius(20)
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| ui.vertical(add_contents).inner)
        .inner
}

/// Full-width (or fixed-width) button with a left-to-right accent gradient.
pub fn gradient_button(ui: &mut egui::Ui, label: &str, enabled: bool, size: Vec2) -> egui::Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = rect.height() / 2.0;
        if enabled {
            let lift = if response.hovered() { 22 } else { 0 };
            let (a, b) = (brighten(ACCENT_A, lift), brighten(ACCENT_B, lift));
            // Pill-shaped fan mesh; each vertex is colored by its x position so the
            // gradient follows the rounded outline exactly.
            let mut outline = rounded_outline(rect, radius, 10);
            outline.pop();
            let color_at = |x: f32| lerp_color(a, b, ((x - rect.left()) / rect.width()).clamp(0.0, 1.0));
            let mut mesh = egui::Mesh::default();
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.center(),
                uv: egui::epaint::WHITE_UV,
                color: color_at(rect.center().x),
            });
            for point in &outline {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: *point,
                    uv: egui::epaint::WHITE_UV,
                    color: color_at(point.x),
                });
            }
            let n = outline.len() as u32;
            for i in 0..n {
                mesh.indices.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % n]);
            }
            painter.add(Shape::mesh(mesh));
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                StrokeKind::Inside,
            );
        } else {
            painter.rect_filled(rect, radius, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
            painter.rect_stroke(rect, radius, glass_stroke(), StrokeKind::Inside);
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            if enabled { Color32::WHITE } else { TEXT_DIM },
        );
    }
    response.on_hover_cursor(if enabled { egui::CursorIcon::PointingHand } else { egui::CursorIcon::NotAllowed })
}

fn brighten(c: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_add(amount),
        c.g().saturating_add(amount),
        c.b().saturating_add(amount),
    )
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}

/// Secondary button: glass fill with a subtle border.
pub fn ghost_button(ui: &mut egui::Ui, label: &str, enabled: bool, size: Vec2) -> egui::Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let painter = ui.painter();
    let radius = rect.height() / 2.0;
    let alpha = if enabled && response.hovered() { 26 } else { 12 };
    painter.rect_filled(rect, radius, Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
    painter.rect_stroke(rect, radius, glass_stroke(), StrokeKind::Inside);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.5),
        if enabled { TEXT } else { TEXT_DIM.gamma_multiply(0.7) },
    );
    response.on_hover_cursor(if enabled { egui::CursorIcon::PointingHand } else { egui::CursorIcon::NotAllowed })
}

/// Small accent pill button (e.g. "Scan").
pub fn accent_pill(ui: &mut egui::Ui, label: &str, size: Vec2, danger: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let radius = rect.height() / 2.0;
    let fill = if danger {
        Color32::from_rgb(140, 40, 48)
    } else if response.hovered() {
        Color32::from_rgb(118, 82, 240)
    } else {
        Color32::from_rgb(98, 64, 220)
    };
    painter.rect_filled(rect, radius, fill);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Two-option segmented control. Returns true if the selection changed.
pub fn segmented(ui: &mut egui::Ui, selected_second: &mut bool, first: &str, second: &str) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 10))
        .stroke(glass_stroke())
        .corner_radius(20)
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.horizontal(|ui| {
                for (is_second, label) in [(false, first), (true, second)] {
                    let active = *selected_second == is_second;
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(86.0, 28.0), Sense::click());
                    if active {
                        ui.painter().rect_filled(rect, 14, Color32::from_rgb(98, 64, 220));
                    } else if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, 14, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
                    }
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(12.5),
                        if active { Color32::WHITE } else { TEXT_DIM },
                    );
                    if response.clicked() && !active {
                        *selected_second = is_second;
                        changed = true;
                    }
                }
            });
        });
    changed
}

fn rounded_outline(rect: Rect, radius: f32, steps_per_corner: usize) -> Vec<Pos2> {
    let mut points = Vec::new();
    let corners = [
        (Pos2::new(rect.right() - radius, rect.top() + radius), -90.0_f32),
        (Pos2::new(rect.right() - radius, rect.bottom() - radius), 0.0),
        (Pos2::new(rect.left() + radius, rect.bottom() - radius), 90.0),
        (Pos2::new(rect.left() + radius, rect.top() + radius), 180.0),
    ];
    for (center, start) in corners {
        for step in 0..=steps_per_corner {
            let angle = (start + step as f32 * 90.0 / steps_per_corner as f32).to_radians();
            points.push(center + Vec2::angled(angle) * radius);
        }
    }
    points.push(points[0]);
    points
}

/// Dashed drop zone. `hovered_file` highlights it while a file is dragged over the window.
pub fn dropzone(
    ui: &mut egui::Ui,
    height: f32,
    hovered_file: bool,
    title: &str,
    link: &str,
    hint: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let active = hovered_file || response.hovered();
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        16,
        if hovered_file {
            Color32::from_rgba_unmultiplied(120, 80, 255, 40)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, if active { 14 } else { 6 })
        },
    );
    let stroke_color = if active { ACCENT_B } else { Color32::from_rgba_unmultiplied(255, 255, 255, 70) };
    painter.extend(Shape::dashed_line(
        &rounded_outline(rect.shrink(1.0), 16.0, 6),
        Stroke::new(1.2_f32, stroke_color),
        6.0,
        5.0,
    ));

    // Upload icon: arrow above a tray.
    let c = Pos2::new(rect.center().x, rect.top() + height * 0.28);
    let icon = Stroke::new(1.6_f32, TEXT_DIM);
    painter.line_segment([c + Vec2::new(0.0, 8.0), c + Vec2::new(0.0, -10.0)], icon);
    painter.line_segment([c + Vec2::new(0.0, -10.0), c + Vec2::new(-6.0, -4.0)], icon);
    painter.line_segment([c + Vec2::new(0.0, -10.0), c + Vec2::new(6.0, -4.0)], icon);
    painter.add(Shape::line(
        vec![
            c + Vec2::new(-11.0, 4.0),
            c + Vec2::new(-11.0, 12.0),
            c + Vec2::new(11.0, 12.0),
            c + Vec2::new(11.0, 4.0),
        ],
        icon,
    ));

    let font = egui::FontId::proportional(13.5);
    let y = rect.top() + height * 0.62;
    let title_galley = painter.layout_no_wrap(title.to_string(), font.clone(), TEXT);
    let link_galley = painter.layout_no_wrap(link.to_string(), font.clone(), ACCENT_B);
    let hint_galley = painter.layout_no_wrap(hint.to_string(), font.clone(), TEXT_DIM);
    let total = title_galley.size().x + 4.0 + link_galley.size().x + 4.0 + hint_galley.size().x;
    let mut x = rect.center().x - total / 2.0;
    let top = y - title_galley.size().y / 2.0;
    let link_rect = Rect::from_min_size(
        Pos2::new(x + title_galley.size().x + 4.0, top),
        link_galley.size(),
    );
    painter.galley(Pos2::new(x, top), title_galley.clone(), TEXT);
    x += title_galley.size().x + 4.0;
    painter.galley(Pos2::new(x, top), link_galley.clone(), ACCENT_B);
    painter.line_segment(
        [
            Pos2::new(link_rect.left(), link_rect.bottom() + 1.0),
            Pos2::new(link_rect.right(), link_rect.bottom() + 1.0),
        ],
        Stroke::new(1.0_f32, ACCENT_B),
    );
    x += link_galley.size().x + 4.0;
    painter.galley(Pos2::new(x, top), hint_galley, TEXT_DIM);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Brand badge text color / background for a payment network label.
pub fn network_color(network: &str) -> Color32 {
    match network {
        "Visa" => Color32::from_rgb(60, 90, 200),
        "Mastercard" => Color32::from_rgb(200, 90, 50),
        "Amex" => Color32::from_rgb(40, 130, 170),
        "Discover" => Color32::from_rgb(210, 130, 40),
        _ => Color32::from_rgb(110, 110, 130),
    }
}

pub struct CardRow<'a> {
    pub title: String,
    pub subtitle: &'a str,
    pub network: Option<&'a str>,
    pub thumb: Option<&'a egui::TextureHandle>,
}

/// Draws a card entry (thumbnail, title, hash). Returns the response of the whole row.
pub fn card_row(
    ui: &mut egui::Ui,
    width: f32,
    row: &CardRow,
    selected: bool,
    chevron: Option<bool>,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 60.0), Sense::click());
    let painter = ui.painter();
    let fill_alpha = if selected || response.hovered() { 22 } else { 12 };
    painter.rect_filled(rect, 14, Color32::from_rgba_unmultiplied(255, 255, 255, fill_alpha));
    painter.rect_stroke(
        rect,
        14,
        if selected { Stroke::new(1.2_f32, ACCENT_B) } else { glass_stroke() },
        StrokeKind::Inside,
    );

    let thumb_rect = Rect::from_min_size(rect.left_top() + Vec2::new(12.0, 13.0), Vec2::new(54.0, 34.0));
    match row.thumb {
        Some(texture) => {
            egui::Image::new(egui::load::SizedTexture::new(texture.id(), thumb_rect.size()))
                .corner_radius(6)
                .paint_at(ui, thumb_rect);
        }
        None => {
            let tint = row.network.map(network_color).unwrap_or(Color32::from_rgb(70, 70, 96));
            painter.rect_filled(thumb_rect, 6, tint);
            painter.rect_filled(
                Rect::from_min_size(thumb_rect.left_top() + Vec2::new(5.0, 9.0), Vec2::new(10.0, 7.0)),
                2,
                Color32::from_rgba_unmultiplied(255, 220, 140, 200),
            );
        }
    }

    let text_x = thumb_rect.right() + 12.0;
    painter.text(
        Pos2::new(text_x, rect.top() + 21.0),
        egui::Align2::LEFT_CENTER,
        &row.title,
        egui::FontId::proportional(14.0),
        TEXT,
    );
    painter.text(
        Pos2::new(text_x, rect.top() + 41.0),
        egui::Align2::LEFT_CENTER,
        row.subtitle,
        egui::FontId::monospace(11.0),
        TEXT_DIM,
    );

    if let Some(open) = chevron {
        let c = Pos2::new(rect.right() - 22.0, rect.center().y);
        let (a, b) = if open { (-3.0, 3.0) } else { (3.0, -3.0) };
        let stroke = Stroke::new(1.6_f32, TEXT_DIM);
        painter.line_segment([c + Vec2::new(-5.0, a), c + Vec2::new(0.0, b)], stroke);
        painter.line_segment([c + Vec2::new(0.0, b), c + Vec2::new(5.0, a)], stroke);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn status_dot(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
    ui.painter()
        .circle_filled(rect.center(), 7.0, color.gamma_multiply(0.18));
    response
}

/// Horizontal zoom slider with "-" and "+" ends. Marks the response as changed when the value moves.
pub fn zoom_slider(ui: &mut egui::Ui, value: &mut f32, min: f32, max: f32, width: f32) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click_and_drag());
    let minus = Rect::from_center_size(Pos2::new(rect.left() + 12.0, rect.center().y), Vec2::splat(24.0));
    let plus = Rect::from_center_size(Pos2::new(rect.right() - 12.0, rect.center().y), Vec2::splat(24.0));
    let track = Rect::from_min_max(
        Pos2::new(minus.right() + 10.0, rect.center().y - 2.0),
        Pos2::new(plus.left() - 10.0, rect.center().y + 2.0),
    );

    let before = *value;
    if let Some(pos) = response.interact_pointer_pos() {
        if response.clicked() && minus.contains(pos) {
            *value = (*value - 0.25).max(min);
        } else if response.clicked() && plus.contains(pos) {
            *value = (*value + 0.25).min(max);
        } else if (response.dragged() || response.clicked()) && pos.x >= track.left() - 6.0 && pos.x <= track.right() + 6.0 {
            let t = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            *value = min + t * (max - min);
        }
    }
    if *value != before {
        response.mark_changed();
    }

    let painter = ui.painter();
    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    painter.rect_filled(track, 2, Color32::from_rgba_unmultiplied(255, 255, 255, 40));
    let filled = Rect::from_min_max(track.min, Pos2::new(track.left() + track.width() * t, track.max.y));
    painter.rect_filled(filled, 2, ACCENT_B);
    let thumb = Pos2::new(track.left() + track.width() * t, rect.center().y);
    painter.circle_filled(thumb, 9.0, Color32::WHITE);
    painter.circle_stroke(thumb, 9.0, Stroke::new(2.0_f32, ACCENT_B));

    let glyph = Stroke::new(1.6_f32, TEXT_DIM);
    painter.line_segment([minus.center() + Vec2::new(-5.0, 0.0), minus.center() + Vec2::new(5.0, 0.0)], glyph);
    painter.line_segment([plus.center() + Vec2::new(-5.0, 0.0), plus.center() + Vec2::new(5.0, 0.0)], glyph);
    painter.line_segment([plus.center() + Vec2::new(0.0, -5.0), plus.center() + Vec2::new(0.0, 5.0)], glyph);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Text shown on the generic Apple Pay style preview screen.
pub struct ScreenTexts<'a> {
    pub hold_near_reader: &'a str,
    pub empty: &'a str,
}

/// Draws a generic "Apple Pay" payment screen with `art` placed on the card slot.
/// `art` is the card texture plus the UV window to show. Nothing here comes from a real
/// user's card; `digits` is the text shown over the bottom-left of the card.
pub fn wallet_screen_preview(
    ui: &mut egui::Ui,
    rect: Rect,
    art: Option<(&egui::TextureHandle, Rect)>,
    digits: &str,
    texts: &ScreenTexts,
) {
    let w = rect.width();
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 26, Color32::from_rgb(242, 242, 247));
    painter.rect_stroke(rect, 26, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 60)), StrokeKind::Inside);

    let margin = w * 0.048;
    let card_w = w - margin * 2.0;
    let card_h = card_w * 969.0 / 1536.0;
    let card = Rect::from_min_size(Pos2::new(rect.left() + margin, rect.top() + w * 0.07), Vec2::new(card_w, card_h));
    let radius = card_w * 0.045;

    painter.add(
        egui::Shadow { offset: [0, 10], blur: 26, spread: 0, color: Color32::from_black_alpha(70) }
            .as_shape(card, radius),
    );
    match art {
        Some((texture, uv)) => {
            egui::Image::new(egui::load::SizedTexture::new(texture.id(), card.size()))
                .uv(uv)
                .corner_radius(radius)
                .paint_at(ui, card);
            let text_pos = Pos2::new(card.left() + card_w * 0.055, card.bottom() - card_h * 0.1);
            let font = egui::FontId::proportional(card_w * 0.052);
            painter.text(text_pos + Vec2::new(0.0, 1.5), egui::Align2::LEFT_CENTER, digits, font.clone(), Color32::from_black_alpha(110));
            painter.text(text_pos, egui::Align2::LEFT_CENTER, digits, font, Color32::WHITE);
        }
        None => {
            painter.rect_filled(card, radius, Color32::from_rgb(222, 222, 230));
            painter.text(card.center(), egui::Align2::CENTER_CENTER, texts.empty, egui::FontId::proportional(card_w * 0.05), Color32::from_rgb(120, 120, 135));
        }
    }

    let blue = Color32::from_rgb(10, 132, 255);
    let center = Pos2::new(rect.center().x, card.bottom() + w * 0.17);
    let r = w * 0.085;
    painter.circle_filled(center, r, Color32::from_rgb(222, 236, 252));
    painter.circle_stroke(center, r, Stroke::new(2.0_f32, blue));
    let phone = Rect::from_center_size(center, Vec2::new(r * 0.5, r * 0.95));
    painter.rect_filled(phone, 4, Color32::WHITE);
    painter.rect_stroke(phone, 4, Stroke::new(2.0_f32, blue), StrokeKind::Inside);
    painter.line_segment(
        [phone.center_top() + Vec2::new(-3.0, 4.0), phone.center_top() + Vec2::new(3.0, 4.0)],
        Stroke::new(1.5_f32, blue),
    );

    painter.text(
        Pos2::new(rect.center().x, center.y + r + w * 0.065),
        egui::Align2::CENTER_CENTER,
        texts.hold_near_reader,
        egui::FontId::proportional(w * 0.052),
        Color32::from_rgb(142, 142, 147),
    );
}

/// Compact outlined pill sized to its text; used for status badges that open a dialog.
pub fn pill_button(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    let font = egui::FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, color);
    let size = Vec2::new(galley.size().x + 26.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let alpha = if response.hovered() { 60 } else { 34 };
    painter.rect_filled(rect, 13, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha));
    painter.rect_stroke(
        rect,
        13,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 140)),
        StrokeKind::Inside,
    );
    painter.galley(Pos2::new(rect.left() + 13.0, rect.center().y - galley.size().y / 2.0), galley, color);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
