use egui::{Align2, Color32, FontFamily, FontId, Image, ImageSource, Rect, Ui};

/// Icon for ribbon buttons.
///
/// Two sources are supported:
/// * a font glyph (icon fonts like Material Symbols, Phosphor, Font Awesome,
///   or plain emoji from egui's built-in fonts);
/// * an image: PNG, SVG and anything else egui's image loaders support
///   (install them with `egui_extras::install_image_loaders`).
#[derive(Clone, Debug)]
pub enum RibbonIcon {
    /// A font glyph. Sized to fit the button
    /// (taking `scale` into account) unless `size` is set explicitly.
    Glyph {
        text: String,
        family: FontFamily,
        size: Option<f32>,
        scale: f32,
        color: Option<Color32>,
    },
    /// An image. `tint` recolors it (handy for monochrome SVGs).
    Image {
        source: ImageSource<'static>,
        tint: Option<Color32>,
    },
}

impl RibbonIcon {
    /// Glyph icon from the default font (`FontFamily::Proportional`).
    pub fn glyph(ch: char) -> Self {
        Self::Glyph {
            text: ch.to_string(),
            family: FontFamily::Proportional,
            size: None,
            scale: 1.0,
            color: None,
        }
    }

    /// Glyph icon from a specific font family,
    /// e.g. `FontFamily::Name("icons".into())`.
    pub fn glyph_with_family(ch: char, family: FontFamily) -> Self {
        Self::glyph(ch).family(family)
    }

    /// Glyph icon with an explicit font (size + family).
    pub fn glyph_font(ch: char, font: FontId) -> Self {
        Self::Glyph {
            text: ch.to_string(),
            family: font.family,
            size: Some(font.size),
            scale: 1.0,
            color: None,
        }
    }

    /// Image icon from any `ImageSource`, e.g.
    /// `egui::include_image!("icons/save.svg")` or `"file://icons/save.png"`.
    pub fn image(source: impl Into<ImageSource<'static>>) -> Self {
        Self::Image {
            source: source.into(),
            tint: None,
        }
    }

    /// Image icon by URI (`file://…`, `https://…`, `bytes://…`).
    pub fn uri(uri: impl Into<String>) -> Self {
        Self::image(ImageSource::Uri(uri.into().into()))
    }

    /// Set the font family of a glyph.
    pub fn family(mut self, new_family: FontFamily) -> Self {
        if let Self::Glyph { family, .. } = &mut self {
            *family = new_family;
        }
        self
    }

    /// Glyph scale relative to the icon size (1.0 by default).
    /// Useful when an icon font has large built-in padding.
    pub fn scale(mut self, new_scale: f32) -> Self {
        if let Self::Glyph { scale, .. } = &mut self {
            *scale = new_scale;
        }
        self
    }

    /// Icon color: text color for a glyph, tint for an image.
    pub fn color(mut self, new_color: Color32) -> Self {
        match &mut self {
            Self::Glyph { color, .. } => *color = Some(new_color),
            Self::Image { tint, .. } => *tint = Some(new_color),
        }
        self
    }

    /// Paint the icon fitted into `rect`.
    /// `fallback_color` is used for glyphs without their own color.
    pub fn paint(&self, ui: &Ui, rect: Rect, fallback_color: Color32) {
        match self {
            Self::Glyph {
                text,
                family,
                size,
                scale,
                color,
            } => {
                let size = size.unwrap_or(rect.height()) * scale;
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    text,
                    FontId::new(size, family.clone()),
                    color.unwrap_or(fallback_color),
                );
            }
            Self::Image { source, tint } => {
                let mut image = Image::new(source.clone()).fit_to_exact_size(rect.size());
                if let Some(tint) = tint {
                    image = image.tint(*tint);
                }
                image.paint_at(ui, rect);
            }
        }
    }
}

impl From<char> for RibbonIcon {
    fn from(ch: char) -> Self {
        Self::glyph(ch)
    }
}

impl From<ImageSource<'static>> for RibbonIcon {
    fn from(source: ImageSource<'static>) -> Self {
        Self::image(source)
    }
}
