use egui::{
    pos2, vec2, Align, Color32, CornerRadius, FontId, Id, InnerResponse, Layout, Pos2,
    Rect, Response, Sense, Shape, Stroke, Ui, UiBuilder,
};

use crate::{RibbonStyle, TabUi};

/// Ribbon state, kept in egui memory between frames.
#[derive(Clone, Debug, Default)]
struct RibbonState {
    /// Selected tab (by title).
    selected: Option<Id>,
    /// Tabs seen in the previous frame.
    tabs: Vec<Id>,
    /// The ribbon is collapsed (only tab headers are visible).
    collapsed: bool,
    /// Horizontal scroll offset of the content.
    offset: f32,
    /// Content width in the previous frame.
    content_width: f32,
}

/// An MS Office-style ribbon: a row of tabs with the selected tab's groups
/// of controls below it. If the groups don't fit the width, the ribbon can be
/// scrolled with the mouse wheel or the arrows at its edges.
///
/// ```ignore
/// Ribbon::new("main_ribbon").show(ui, |ribbon| {
///     ribbon.tab("Home", |tab| {
///         tab.group("Clipboard", |g| {
///             if g.large_button('📋', "Paste").clicked() { /* … */ }
///             g.small_button('✂', "Cut");
///             g.small_button('🗐', "Copy");
///         });
///     });
/// });
/// ```
pub struct Ribbon {
    id_salt: Id,
    style: RibbonStyle,
}

impl Ribbon {
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            id_salt: Id::new(id_salt),
            style: RibbonStyle::default(),
        }
    }

    pub fn style(mut self, style: RibbonStyle) -> Self {
        self.style = style;
        self
    }

    /// Select a tab by title programmatically.
    /// `id_salt` must match the one passed to [`Ribbon::new`].
    pub fn select_tab(ctx: &egui::Context, id_salt: impl std::hash::Hash + std::fmt::Debug, title: &str) {
        ctx.data_mut(|d| {
            let state = d.get_temp_mut_or_default::<RibbonState>(Id::new(id_salt));
            state.selected = Some(tab_id(title));
            state.collapsed = false;
        });
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut RibbonUi) -> R,
    ) -> InnerResponse<R> {
        let Self { id_salt, style } = self;
        // The id is global so that select_tab works from anywhere.
        let id = id_salt;
        let mut state: RibbonState = ui.data(|d| d.get_temp(id)).unwrap_or_default();

        // If the selected tab is gone, fall back to the first one.
        if !state.tabs.is_empty() && state.selected.is_some_and(|s| !state.tabs.contains(&s)) {
            state.selected = None;
        }
        if state.selected.is_none() {
            state.selected = state.tabs.first().copied();
        }

        let width = ui.available_width();
        let height = style.tab_height + if state.collapsed { 0.0 } else { style.body_height() };
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        let header_rect = Rect::from_min_size(rect.min, vec2(width, style.tab_height));
        let body_rect = Rect::from_min_max(pos2(rect.left(), header_rect.bottom()), rect.max);

        if !state.collapsed {
            ui.painter()
                .rect_filled(body_rect, 0.0, style.body_fill_color(ui));
        }
        let border = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter().hline(rect.x_range(), header_rect.bottom() - 0.5, border);
        if !state.collapsed {
            ui.painter().hline(rect.x_range(), body_rect.bottom() - 0.5, border);
        }

        let mut header = ui.new_child(
            UiBuilder::new()
                .id_salt((id_salt, "header"))
                .max_rect(header_rect.shrink2(vec2(4.0, 0.0)))
                .layout(Layout::left_to_right(Align::Max)),
        );
        header.spacing_mut().item_spacing.x = 2.0;

        let viewport = body_rect.shrink2(vec2(0.0, 1.0));
        let max_offset = (state.content_width - viewport.width()).max(0.0);
        let overflow = max_offset > 0.5;
        let max_offset = if overflow {
            max_offset + style.scroll_button_width
        } else {
            0.0
        };

        // The mouse wheel (both vertical and horizontal) scrolls the ribbon.
        if overflow && !state.collapsed && ui.rect_contains_pointer(viewport) {
            let delta = ui.input(|i| i.smooth_scroll_delta);
            let d = delta.x + delta.y;
            if d != 0.0 {
                state.offset -= d;
                ui.input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
            }
        }
        state.offset = state.offset.clamp(0.0, max_offset);

        let body = (!state.collapsed).then(|| {
            let content_rect = Rect::from_min_size(
                pos2(
                    viewport.left() - state.offset.round(),
                    viewport.top() + style.body_padding,
                ),
                vec2(100_000.0, style.content_height + style.caption_height),
            );
            let mut body = ui.new_child(
                UiBuilder::new()
                    .id_salt((id_salt, "body"))
                    .max_rect(content_rect)
                    .layout(Layout::left_to_right(Align::Min)),
            );
            body.set_clip_rect(viewport.intersect(ui.clip_rect()));
            body.spacing_mut().item_spacing = vec2(0.0, 0.0);
            body
        });

        let mut ribbon = RibbonUi {
            style,
            state,
            header,
            body,
            seen_tabs: Vec::new(),
        };
        let inner = add_contents(&mut ribbon);

        let RibbonUi {
            style,
            mut state,
            body,
            seen_tabs,
            ..
        } = ribbon;
        state.tabs = seen_tabs;

        if let Some(mut body) = body {
            state.content_width = body.min_rect().width();
            scroll_buttons(&mut body, id, &style, viewport, overflow, max_offset, &mut state);
        }

        ui.data_mut(|d| d.insert_temp(id, state));
        InnerResponse::new(inner, response)
    }
}

fn tab_id(title: &str) -> Id {
    Id::new(("ribbon_tab", title))
}

/// Scroll arrows at the ribbon edges: scrolling continues while an arrow is held.
fn scroll_buttons(
    body: &mut Ui,
    id: Id,
    style: &RibbonStyle,
    viewport: Rect,
    overflow: bool,
    max_offset: f32,
    state: &mut RibbonState,
) {
    if !overflow {
        return;
    }
    let w = style.scroll_button_width;
    let dt = body.input(|i| i.stable_dt).min(0.1);
    let buttons = [
        (
            -1.0,
            Rect::from_min_size(viewport.min, vec2(w, viewport.height())),
        ),
        (
            1.0,
            Rect::from_min_size(
                pos2(viewport.right() - w, viewport.top()),
                vec2(w, viewport.height()),
            ),
        ),
    ];
    for (dir, rect) in buttons {
        let visible = if dir < 0.0 {
            state.offset > 0.5
        } else {
            state.offset < max_offset - 0.5
        };
        if !visible {
            continue;
        }
        let resp = body.interact(rect, id.with(("scroll", dir as i32)), Sense::click_and_drag());
        let visuals = body.visuals();
        let w = if resp.is_pointer_button_down_on() {
            &visuals.widgets.active
        } else if resp.hovered() {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
        body.painter()
            .rect_filled(rect, CornerRadius::ZERO, w.weak_bg_fill);
        body.painter().vline(
            if dir < 0.0 { rect.right() - 0.5 } else { rect.left() + 0.5 },
            rect.y_range(),
            visuals.widgets.noninteractive.bg_stroke,
        );
        paint_side_arrow(body, rect.center(), dir, w.fg_stroke.color);

        if resp.is_pointer_button_down_on() {
            state.offset = (state.offset + dir * style.scroll_speed * dt).clamp(0.0, max_offset);
            body.ctx().request_repaint();
        }
    }
}

fn paint_side_arrow(ui: &Ui, c: Pos2, dir: f32, color: Color32) {
    let s = 4.0;
    ui.painter().add(Shape::convex_polygon(
        vec![
            pos2(c.x - dir * s * 0.5, c.y - s),
            pos2(c.x + dir * s * 0.7, c.y),
            pos2(c.x - dir * s * 0.5, c.y + s),
        ],
        color,
        Stroke::NONE,
    ));
}

/// Ribbon builder passed to the [`Ribbon::show`] closure.
pub struct RibbonUi {
    style: RibbonStyle,
    state: RibbonState,
    header: Ui,
    body: Option<Ui>,
    seen_tabs: Vec<Id>,
}

impl RibbonUi {
    /// The ribbon style.
    pub fn style(&self) -> &RibbonStyle {
        &self.style
    }

    /// Whether the tab with this title is currently selected.
    pub fn is_selected(&self, title: &str) -> bool {
        self.state.selected == Some(tab_id(title))
    }

    /// Whether the ribbon is collapsed.
    pub fn is_collapsed(&self) -> bool {
        self.state.collapsed
    }

    /// Accent button left of the tabs (like "File" in Office) with a dropdown menu.
    pub fn app_menu(&mut self, label: &str, add_contents: impl FnOnce(&mut Ui)) -> Response {
        let style = &self.style;
        let font = FontId::proportional(style.tab_font_size);
        let galley = self
            .header
            .painter()
            .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
        let size = vec2(galley.size().x + 24.0, style.tab_height - 3.0);
        let (rect, response) = self.header.allocate_exact_size(size, Sense::click());
        let accent = style.accent_color(&self.header);
        let fill = if response.hovered() {
            accent.gamma_multiply(1.15)
        } else {
            accent
        };
        self.header.painter().rect_filled(
            rect,
            CornerRadius {
                nw: style.corner_radius,
                ne: style.corner_radius,
                sw: 0,
                se: 0,
            },
            fill,
        );
        // Pick the text color based on the accent brightness.
        let [r, g, b, _] = fill.to_array();
        let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        let text_color = if luma > 150.0 {
            Color32::from_gray(20)
        } else {
            Color32::WHITE
        };
        self.header.painter().galley(
            rect.center() - galley.size() / 2.0,
            galley,
            text_color,
        );
        egui::Popup::menu(&response).show(add_contents);
        response
    }

    /// A tab. The contents closure only runs for the selected tab.
    ///
    /// Clicking a tab selects it; double-clicking collapses or expands the ribbon.
    pub fn tab(&mut self, title: &str, add_contents: impl FnOnce(&mut TabUi<'_>)) -> Response {
        let id = tab_id(title);
        self.seen_tabs.push(id);
        if self.state.selected.is_none() {
            self.state.selected = Some(id);
        }
        let selected = self.state.selected == Some(id);

        let response = self.tab_header(title, selected);
        if response.double_clicked() {
            self.state.collapsed = !self.state.collapsed;
            response.ctx.request_repaint();
        } else if response.clicked() {
            self.state.collapsed = false;
            self.state.selected = Some(id);
            response.ctx.request_repaint();
        }

        if let (true, Some(body)) = (selected, &mut self.body) {
            let mut tab = TabUi {
                ui: body,
                style: &self.style,
            };
            add_contents(&mut tab);
        }
        response
    }

    fn tab_header(&mut self, title: &str, selected: bool) -> Response {
        let style = &self.style;
        let ui = &mut self.header;
        let font = FontId::proportional(style.tab_font_size);
        let color = if selected {
            ui.visuals().strong_text_color()
        } else {
            ui.visuals().text_color()
        };
        let galley = ui.painter().layout_no_wrap(title.to_owned(), font, color);
        let size = vec2(galley.size().x + 20.0, style.tab_height - 3.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, title)
        });

        if response.hovered() && !selected {
            ui.painter().rect_filled(
                rect,
                CornerRadius {
                    nw: style.corner_radius,
                    ne: style.corner_radius,
                    sw: 0,
                    se: 0,
                },
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }
        let text_pos = rect.center() - galley.size() / 2.0;
        ui.painter().galley(text_pos, galley.clone(), color);
        if selected && !self.state.collapsed {
            let accent = style.accent_color(ui);
            let underline = Rect::from_center_size(
                pos2(rect.center().x, rect.bottom() - 2.0),
                vec2(galley.size().x.max(16.0), 3.0),
            );
            ui.painter().rect_filled(underline, 1.5, accent);
        }
        response
    }
}
