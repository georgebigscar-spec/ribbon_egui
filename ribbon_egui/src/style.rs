use egui::{Color32, Ui};

/// Ribbon sizes and colors. Colors default to the current egui theme.
#[derive(Clone, Debug)]
pub struct RibbonStyle {
    /// Height of the tab row.
    pub tab_height: f32,
    /// Height of the controls area inside a group (three rows of small buttons).
    pub content_height: f32,
    /// Height of a group caption.
    pub caption_height: f32,
    /// Top and bottom padding inside the ribbon body.
    pub body_padding: f32,
    /// Horizontal padding inside a group.
    pub group_padding: f32,
    /// Spacing between controls inside a group.
    pub item_spacing: f32,
    /// Icon size of a large button.
    pub large_icon_size: f32,
    /// Icon size of a small button.
    pub small_icon_size: f32,
    /// Font size of button labels.
    pub label_font_size: f32,
    /// Font size of tab titles.
    pub tab_font_size: f32,
    /// Font size of group captions.
    pub caption_font_size: f32,
    /// Width of the scroll arrows.
    pub scroll_button_width: f32,
    /// Scroll speed while an arrow is held, in points per second.
    pub scroll_speed: f32,
    /// Button corner radius.
    pub corner_radius: u8,
    /// Accent color (active tab underline, application button).
    pub accent: Option<Color32>,
    /// Ribbon body background.
    pub body_fill: Option<Color32>,
}

impl Default for RibbonStyle {
    fn default() -> Self {
        Self {
            tab_height: 26.0,
            content_height: 72.0,
            caption_height: 16.0,
            body_padding: 3.0,
            group_padding: 4.0,
            item_spacing: 2.0,
            large_icon_size: 32.0,
            small_icon_size: 16.0,
            label_font_size: 12.0,
            tab_font_size: 13.0,
            caption_font_size: 11.0,
            scroll_button_width: 16.0,
            scroll_speed: 600.0,
            corner_radius: 4,
            accent: None,
            body_fill: None,
        }
    }
}

impl RibbonStyle {
    /// Height of one row of small controls.
    pub fn row_height(&self) -> f32 {
        self.content_height / 3.0
    }

    /// Full height of the ribbon body (excluding the tab row).
    pub fn body_height(&self) -> f32 {
        self.body_padding * 2.0 + self.content_height + self.caption_height
    }

    pub(crate) fn accent_color(&self, ui: &Ui) -> Color32 {
        self.accent.unwrap_or(ui.visuals().selection.bg_fill)
    }

    pub(crate) fn body_fill_color(&self, ui: &Ui) -> Color32 {
        self.body_fill.unwrap_or(ui.visuals().window_fill)
    }
}
