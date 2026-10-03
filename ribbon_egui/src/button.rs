use std::sync::Arc;

use egui::{
    pos2, vec2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui,
};

use crate::{RibbonIcon, RibbonStyle};

/// Size of a ribbon button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    /// Large button spanning the full group height: icon on top, label below.
    Large,
    /// Small single-row button: icon on the left, label on the right.
    /// Small buttons are automatically stacked into columns of three.
    Small,
}

/// A ribbon button description. Add it with [`crate::GroupUi::button`].
///
/// ```ignore
/// group.button(
///     RibbonButton::large("Paste")
///         .icon(RibbonIcon::glyph('📋'))
///         .selected(is_on)
///         .enabled(can_paste),
/// );
/// ```
#[derive(Clone, Debug)]
pub struct RibbonButton {
    pub(crate) size: ButtonSize,
    pub(crate) label: String,
    pub(crate) icon: Option<RibbonIcon>,
    pub(crate) selected: bool,
    pub(crate) enabled: bool,
    pub(crate) dropdown: bool,
}

impl RibbonButton {
    pub fn new(size: ButtonSize, label: impl Into<String>) -> Self {
        Self {
            size,
            label: label.into(),
            icon: None,
            selected: false,
            enabled: true,
            dropdown: false,
        }
    }

    pub fn large(label: impl Into<String>) -> Self {
        Self::new(ButtonSize::Large, label)
    }

    pub fn small(label: impl Into<String>) -> Self {
        Self::new(ButtonSize::Small, label)
    }

    pub fn icon(mut self, icon: impl Into<RibbonIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Pressed (checked) state, for toggle buttons.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Draw a dropdown arrow.
    pub fn dropdown(mut self, dropdown: bool) -> Self {
        self.dropdown = dropdown;
        self
    }

    pub(crate) fn show(self, ui: &mut Ui, style: &RibbonStyle) -> Response {
        let enabled = self.enabled;
        let label = self.label.clone();
        let selected = self.selected;
        let response = ui
            .add_enabled_ui(enabled, |ui| match self.size {
                ButtonSize::Large => self.show_large(ui, style),
                ButtonSize::Small => self.show_small(ui, style),
            })
            .inner;
        response.widget_info(|| {
            if selected {
                egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, true, &label)
            } else {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &label)
            }
        });
        response
    }

    fn show_large(self, ui: &mut Ui, style: &RibbonStyle) -> Response {
        let font = FontId::proportional(style.label_font_size);
        let text_color = ui.visuals().text_color();
        let lines = layout_two_lines(ui, &self.label, font, text_color);
        let arrow_w = if self.dropdown { 10.0 } else { 0.0 };
        let pad = 5.0;

        let line_w = |i: usize| lines[i].size().x + if i + 1 == lines.len() { arrow_w } else { 0.0 };
        let text_w = (0..lines.len()).map(line_w).fold(0.0, f32::max);
        let width = (text_w.max(style.large_icon_size) + pad * 2.0).max(style.large_icon_size + 12.0);
        let (rect, response) =
            ui.allocate_exact_size(vec2(width.ceil(), style.content_height), Sense::click());

        if ui.is_rect_visible(rect) {
            let fg = paint_background(ui, style, &response, rect, self.selected);

            let icon_rect = Rect::from_center_size(
                pos2(rect.center().x, rect.top() + 4.0 + style.large_icon_size / 2.0),
                egui::Vec2::splat(style.large_icon_size),
            );
            if let Some(icon) = &self.icon {
                icon.paint(ui, icon_rect, fg);
            }

            let mut y = icon_rect.bottom() + 3.0;
            for (i, line) in lines.iter().enumerate() {
                let w = line_w(i);
                let pos = pos2((rect.center().x - w / 2.0).round(), y);
                let h = line.size().y;
                ui.painter().galley(pos, line.clone(), fg);
                if self.dropdown && i + 1 == lines.len() {
                    paint_arrow(ui, pos2(pos.x + line.size().x + 6.0, y + h / 2.0), fg);
                }
                y += h;
            }
        }
        response
    }

    fn show_small(self, ui: &mut Ui, style: &RibbonStyle) -> Response {
        let font = FontId::proportional(style.label_font_size);
        let text_color = ui.visuals().text_color();
        let galley = (!self.label.is_empty())
            .then(|| ui.painter().layout_no_wrap(self.label.clone(), font, text_color));
        let pad = 4.0;
        let gap = 4.0;
        let icon_w = if self.icon.is_some() { style.small_icon_size } else { 0.0 };
        let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
        let gap_w = if icon_w > 0.0 && text_w > 0.0 { gap } else { 0.0 };
        let arrow_w = if self.dropdown { 12.0 } else { 0.0 };
        let width = pad + icon_w + gap_w + text_w + arrow_w + pad;
        let (rect, response) =
            ui.allocate_exact_size(vec2(width.ceil(), style.row_height()), Sense::click());

        if ui.is_rect_visible(rect) {
            let fg = paint_background(ui, style, &response, rect, self.selected);
            let mut x = rect.left() + pad;
            if let Some(icon) = &self.icon {
                let icon_rect = Rect::from_min_size(
                    pos2(x, rect.center().y - style.small_icon_size / 2.0),
                    egui::Vec2::splat(style.small_icon_size),
                );
                icon.paint(ui, icon_rect, fg);
                x += icon_w + gap_w;
            }
            if let Some(galley) = galley {
                let pos = pos2(x, rect.center().y - galley.size().y / 2.0);
                ui.painter().galley(pos, galley, fg);
                x += text_w;
            }
            if self.dropdown {
                paint_arrow(ui, pos2(x + 6.0, rect.center().y), fg);
            }
        }
        response
    }
}

/// Large button label: long text is wrapped onto two lines
/// at the space that gives the narrowest button. Each line is centered separately.
fn layout_two_lines(ui: &Ui, label: &str, font: FontId, color: Color32) -> Vec<Arc<Galley>> {
    let painter = ui.painter();
    let layout = |s: &str| painter.layout_no_wrap(s.to_owned(), font.clone(), color);
    let single = layout(label);
    if !label.contains(' ') || single.size().x < 48.0 {
        return vec![single];
    }
    let best = label
        .match_indices(' ')
        .map(|(i, _)| {
            let lines = vec![layout(&label[..i]), layout(&label[i + 1..])];
            let w = lines[0].size().x.max(lines[1].size().x);
            (lines, w)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1));
    match best {
        Some((lines, w)) if w < single.size().x => lines,
        _ => vec![single],
    }
}

fn paint_background(
    ui: &Ui,
    style: &RibbonStyle,
    response: &Response,
    rect: Rect,
    selected: bool,
) -> Color32 {
    let visuals = ui.visuals();
    let radius = CornerRadius::same(style.corner_radius);
    let accent = style.accent_color(ui);
    if selected {
        ui.painter().rect(
            rect,
            radius,
            accent.gamma_multiply(0.35),
            Stroke::new(1.0, accent),
            StrokeKind::Inside,
        );
    }
    if response.is_pointer_button_down_on() {
        let w = &visuals.widgets.active;
        ui.painter()
            .rect(rect, radius, w.weak_bg_fill, w.bg_stroke, StrokeKind::Inside);
        w.fg_stroke.color
    } else if response.hovered() {
        let w = &visuals.widgets.hovered;
        ui.painter()
            .rect(rect, radius, w.weak_bg_fill, w.bg_stroke, StrokeKind::Inside);
        w.fg_stroke.color
    } else if !ui.is_enabled() {
        visuals.widgets.noninteractive.fg_stroke.color
    } else {
        visuals.text_color()
    }
}

/// Small downward triangle centered at `center`.
pub(crate) fn paint_arrow(ui: &Ui, center: Pos2, color: Color32) {
    let s = 3.0;
    ui.painter().add(Shape::convex_polygon(
        vec![
            pos2(center.x - s, center.y - s * 0.5),
            pos2(center.x + s, center.y - s * 0.5),
            pos2(center.x, center.y + s * 0.6),
        ],
        color,
        Stroke::NONE,
    ));
}
