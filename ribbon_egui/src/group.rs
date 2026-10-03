use egui::{
    pos2, vec2, Align, FontId, Layout, Rect, Response, Sense, Stroke, Ui, UiBuilder,
    Vec2, Widget,
};

use crate::{ButtonSize, RibbonButton, RibbonIcon, RibbonStyle};

/// Horizontal bound for child `Ui`s:
/// the actual group width is determined by its content.
const UNBOUNDED_WIDTH: f32 = 100_000.0;

/// Contents of a single tab: a sequence of groups.
pub struct TabUi<'a> {
    pub(crate) ui: &'a mut Ui,
    pub(crate) style: &'a RibbonStyle,
}

/// Result of adding a group.
pub struct GroupResponse {
    /// Rectangle of the whole group, including the caption.
    pub rect: Rect,
    /// The dialog launcher in the caption corner was clicked
    /// (only for [`TabUi::group_with_launcher`]).
    pub launcher_clicked: bool,
}

impl TabUi<'_> {
    /// Add a group of controls with a caption below.
    pub fn group(&mut self, title: &str, add_contents: impl FnOnce(&mut GroupUi)) -> GroupResponse {
        self.group_impl(title, false, add_contents)
    }

    /// A group with a small "↘" button in the caption corner, like Office uses
    /// to open the group's settings dialog.
    pub fn group_with_launcher(
        &mut self,
        title: &str,
        add_contents: impl FnOnce(&mut GroupUi),
    ) -> GroupResponse {
        self.group_impl(title, true, add_contents)
    }

    /// Direct access to the tab's `Ui` for custom drawing.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    fn group_impl(
        &mut self,
        title: &str,
        launcher: bool,
        add_contents: impl FnOnce(&mut GroupUi),
    ) -> GroupResponse {
        let style = self.style;
        let group_id = self.ui.id().with(("ribbon_group", title));
        let pad = style.group_padding;
        let top_left = self.ui.cursor().min;

        // Caption and launcher button.
        let caption_font = FontId::proportional(style.caption_font_size);
        let caption_color = self.ui.visuals().weak_text_color();
        let caption = self
            .ui
            .painter()
            .layout_no_wrap(title.to_owned(), caption_font, caption_color);
        let launcher_w = if launcher { style.caption_height } else { 0.0 };
        let caption_w = caption.size().x + launcher_w * 2.0;

        // If the content was narrower than the caption last frame, center it.
        let prev_content_w: Option<f32> = self.ui.data(|d| d.get_temp(group_id));
        let shift = prev_content_w.map_or(0.0, |w| ((caption_w - w) / 2.0).max(0.0));

        let content_rect = Rect::from_min_size(
            top_left + vec2(pad + shift, 0.0),
            vec2(UNBOUNDED_WIDTH, style.content_height),
        );
        let mut child = self.ui.new_child(
            UiBuilder::new()
                .id_salt(("ribbon_group", title))
                .max_rect(content_rect)
                .layout(Layout::left_to_right(Align::Min)),
        );
        child.spacing_mut().item_spacing = vec2(style.item_spacing, 0.0);

        let mut group = GroupUi::new(child, style, false);
        add_contents(&mut group);
        group.close_column();
        let content_w = group.ui.min_rect().width().max(0.0);
        self.ui.data_mut(|d| d.insert_temp(group_id, content_w));

        let inner_w = content_w.max(caption_w);
        let group_rect = Rect::from_min_size(
            top_left,
            vec2(inner_w + pad * 2.0, style.content_height + style.caption_height),
        );

        let painter = self.ui.painter().clone();
        let caption_pos = pos2(
            group_rect.center().x - caption.size().x / 2.0,
            group_rect.bottom() - style.caption_height / 2.0 - caption.size().y / 2.0,
        );
        painter.galley(caption_pos, caption, caption_color);

        // Separator on the right.
        let sep_x = group_rect.right() + 0.5;
        painter.line_segment(
            [
                pos2(sep_x, group_rect.top() + 2.0),
                pos2(sep_x, group_rect.bottom() - 2.0),
            ],
            self.ui.visuals().widgets.noninteractive.bg_stroke,
        );

        let mut launcher_clicked = false;
        if launcher {
            let r = Rect::from_min_size(
                pos2(
                    group_rect.right() - launcher_w,
                    group_rect.bottom() - style.caption_height,
                ),
                Vec2::splat(launcher_w),
            );
            let resp = self.ui.interact(r, group_id.with("launcher"), Sense::click());
            let color = if resp.hovered() {
                self.ui.visuals().strong_text_color()
            } else {
                caption_color
            };
            if resp.hovered() {
                painter.rect_filled(
                    r.shrink(1.0),
                    style.corner_radius,
                    self.ui.visuals().widgets.hovered.weak_bg_fill,
                );
            }
            paint_launcher_icon(&painter, r.shrink(4.0), color);
            launcher_clicked = resp.clicked();
        }

        self.ui
            .advance_cursor_after_rect(group_rect.expand2(vec2(1.0, 0.0)));
        GroupResponse {
            rect: group_rect,
            launcher_clicked,
        }
    }
}

fn paint_launcher_icon(painter: &egui::Painter, r: Rect, color: egui::Color32) {
    let stroke = Stroke::new(1.0, color);
    // Corner and diagonal arrow, like in Office.
    painter.line_segment([r.left_top(), r.right_top()], stroke);
    painter.line_segment([r.left_top(), r.left_bottom()], stroke);
    painter.line_segment([r.center(), r.right_bottom()], stroke);
    painter.line_segment([r.right_bottom(), pos2(r.right() - 3.0, r.bottom())], stroke);
    painter.line_segment([r.right_bottom(), pos2(r.right(), r.bottom() - 3.0)], stroke);
}

/// Contents of a group.
///
/// Large items take the full group height. Small ones (buttons, checkboxes,
/// arbitrary widgets via [`GroupUi::add`]) are automatically stacked
/// into columns of three rows. Use [`GroupUi::column`] for an explicit column.
pub struct GroupUi<'a> {
    ui: Ui,
    style: &'a RibbonStyle,
    in_column: bool,
    auto_column: Option<(Ui, usize)>,
    column_counter: usize,
}

impl<'a> GroupUi<'a> {
    fn new(ui: Ui, style: &'a RibbonStyle, in_column: bool) -> Self {
        Self {
            ui,
            style,
            in_column,
            auto_column: None,
            column_counter: 0,
        }
    }

    /// The ribbon style.
    pub fn style(&self) -> &RibbonStyle {
        self.style
    }

    /// Add a button with full control over its options.
    pub fn button(&mut self, button: RibbonButton) -> Response {
        let style = self.style;
        let ui = match button.size {
            ButtonSize::Large => self.large_slot(),
            ButtonSize::Small => self.small_slot(),
        };
        button.show(ui, style)
    }

    /// Large button: icon on top, label below.
    pub fn large_button(&mut self, icon: impl Into<RibbonIcon>, label: &str) -> Response {
        self.button(RibbonButton::large(label).icon(icon))
    }

    /// Small button: icon and label in one row.
    pub fn small_button(&mut self, icon: impl Into<RibbonIcon>, label: &str) -> Response {
        self.button(RibbonButton::small(label).icon(icon))
    }

    /// Small icon-only button (add a tooltip with `on_hover_text`).
    pub fn icon_button(&mut self, icon: impl Into<RibbonIcon>) -> Response {
        self.button(RibbonButton::small("").icon(icon))
    }

    /// Large toggle button.
    pub fn large_toggle(
        &mut self,
        value: &mut bool,
        icon: impl Into<RibbonIcon>,
        label: &str,
    ) -> Response {
        let mut r = self.button(RibbonButton::large(label).icon(icon).selected(*value));
        if r.clicked() {
            *value = !*value;
            r.mark_changed();
        }
        r
    }

    /// Small toggle button.
    pub fn small_toggle(
        &mut self,
        value: &mut bool,
        icon: impl Into<RibbonIcon>,
        label: &str,
    ) -> Response {
        let mut r = self.button(RibbonButton::small(label).icon(icon).selected(*value));
        if r.clicked() {
            *value = !*value;
            r.mark_changed();
        }
        r
    }

    /// Large button with a dropdown menu.
    pub fn large_menu(
        &mut self,
        icon: impl Into<RibbonIcon>,
        label: &str,
        add_contents: impl FnOnce(&mut Ui),
    ) -> Response {
        self.menu(RibbonButton::large(label).icon(icon), add_contents)
    }

    /// Small button with a dropdown menu.
    pub fn small_menu(
        &mut self,
        icon: impl Into<RibbonIcon>,
        label: &str,
        add_contents: impl FnOnce(&mut Ui),
    ) -> Response {
        self.menu(RibbonButton::small(label).icon(icon), add_contents)
    }

    /// Any button with a dropdown menu (the arrow is added automatically).
    pub fn menu(&mut self, button: RibbonButton, add_contents: impl FnOnce(&mut Ui)) -> Response {
        let response = self.button(button.dropdown(true));
        egui::Popup::menu(&response).show(add_contents);
        response
    }

    /// An arbitrary small widget (checkbox, combo box, text edit…)
    /// in a single column row.
    pub fn add(&mut self, widget: impl Widget) -> Response {
        self.add_row(|ui| ui.add(widget)).inner
    }

    /// A column row with arbitrary content (several widgets side by side).
    pub fn add_row<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        let row_h = self.style.row_height();
        let spacing = self.style.item_spacing;
        let ui = self.small_slot();
        ui.allocate_ui_with_layout(
            vec2(UNBOUNDED_WIDTH, row_h),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = spacing.max(4.0);
                ui.set_min_height(row_h);
                add_contents(ui)
            },
        )
    }

    /// A row of small buttons, e.g. "B I U" in the Font group.
    /// Takes one column row.
    pub fn row(&mut self, add_contents: impl FnOnce(&mut GroupUi)) {
        let style = self.style;
        let slot = self.small_slot();
        let rect = Rect::from_min_size(slot.cursor().min, vec2(UNBOUNDED_WIDTH, style.row_height()));
        let mut child = slot.new_child(
            UiBuilder::new()
                .id_salt(("ribbon_row", slot.next_auto_id()))
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        child.spacing_mut().item_spacing = Vec2::ZERO;
        let mut row = GroupUi::new(child, style, true);
        add_contents(&mut row);
        let mut used = row.ui.min_rect();
        used.max.y = used.min.y + style.row_height();
        slot.advance_cursor_after_rect(used);
    }

    /// Arbitrary content spanning the full group height.
    pub fn add_large<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        let h = self.style.content_height;
        let ui = self.large_slot();
        ui.allocate_ui_with_layout(vec2(UNBOUNDED_WIDTH, h), Layout::top_down(Align::Min), |ui| {
            ui.set_min_height(h);
            add_contents(ui)
        })
    }

    /// An explicit column of small items (up to three rows).
    pub fn column(&mut self, add_contents: impl FnOnce(&mut GroupUi)) {
        self.close_column();
        let child = self.new_column_ui();
        let mut column = GroupUi::new(child, self.style, true);
        add_contents(&mut column);
        let rect = column.ui.min_rect();
        self.ui.advance_cursor_after_rect(rect);
    }

    /// Vertical separator inside a group.
    pub fn separator(&mut self) {
        let h = self.style.content_height;
        let ui = self.large_slot();
        let (rect, _) = ui.allocate_exact_size(vec2(5.0, h), Sense::hover());
        let x = rect.center().x.round() + 0.5;
        ui.painter().line_segment(
            [pos2(x, rect.top() + 4.0), pos2(x, rect.bottom() - 4.0)],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );
    }

    /// `Ui` for the next "large" item. Draw anything you like in it.
    pub fn ui(&mut self) -> &mut Ui {
        self.large_slot()
    }

    fn large_slot(&mut self) -> &mut Ui {
        self.close_column();
        &mut self.ui
    }

    fn small_slot(&mut self) -> &mut Ui {
        if self.in_column {
            return &mut self.ui;
        }
        if matches!(self.auto_column, Some((_, n)) if n >= 3) {
            self.close_column();
        }
        if self.auto_column.is_none() {
            let child = self.new_column_ui();
            self.auto_column = Some((child, 0));
        }
        let (ui, n) = self.auto_column.as_mut().expect("column was just opened");
        *n += 1;
        ui
    }

    fn new_column_ui(&mut self) -> Ui {
        self.column_counter += 1;
        let rect = Rect::from_min_size(
            self.ui.cursor().min,
            vec2(UNBOUNDED_WIDTH, self.style.content_height),
        );
        let mut child = self.ui.new_child(
            UiBuilder::new()
                .id_salt(("ribbon_column", self.column_counter))
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        child.spacing_mut().item_spacing = Vec2::ZERO;
        child
    }

    pub(crate) fn close_column(&mut self) {
        if let Some((column, _)) = self.auto_column.take() {
            let rect = column.min_rect();
            self.ui.advance_cursor_after_rect(rect);
        }
    }
}
