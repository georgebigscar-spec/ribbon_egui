//! ribbon_egui demo.
//!
//! Run with `cargo run -p demo`.
//! Environment variables for automated screenshots:
//! `RIBBON_SCREENSHOT=out.png` saves a window screenshot and exits,
//! `RIBBON_WIDTH=700` sets the window width, `RIBBON_DARK=1` enables the dark theme,
//! `RIBBON_TAB=Insert` opens a tab.

use eframe::egui::{self, Color32, FontFamily, FontId};
use ribbon_egui::{Ribbon, RibbonButton, RibbonIcon};

fn main() -> eframe::Result {
    let width: f32 = std::env::var("RIBBON_WIDTH")
        .ok()
        .and_then(|w| w.parse().ok())
        .unwrap_or(1400.0);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, 360.0])
            .with_title("ribbon_egui demo"),
        ..Default::default()
    };
    eframe::run_native(
        "ribbon_egui demo",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            setup_fonts(&cc.egui_ctx);
            let theme = if std::env::var("RIBBON_DARK").is_ok() {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            };
            cc.egui_ctx.set_theme(theme);
            Ok(Box::new(DemoApp::default()))
        }),
    )
}

/// Example of registering an icon font under its own family name.
/// For simplicity, egui's built-in pictogram font is registered here as `icons`.
/// Material Symbols, Phosphor etc. are added the same way:
/// `fonts.font_data.insert("material".into(), FontData::from_static(include_bytes!(…)).into())`.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.families.insert(
        FontFamily::Name("icons".into()),
        vec!["emoji-icon-font".into(), "NotoEmoji-Regular".into()],
    );
    ctx.set_fonts(fonts);
}

fn icon_font(ch: char) -> RibbonIcon {
    RibbonIcon::glyph_with_family(ch, FontFamily::Name("icons".into()))
}

fn letter(ch: char) -> RibbonIcon {
    RibbonIcon::glyph_font(ch, FontId::new(15.0, FontFamily::Monospace))
}

struct DemoApp {
    bold: bool,
    italic: bool,
    underline: bool,
    ruler: bool,
    grid: bool,
    navigation: bool,
    zoom: f32,
    font: String,
    font_size: f32,
    align: usize,
    log: Vec<String>,
    frame: usize,
    screenshot_requested: bool,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            bold: true,
            italic: false,
            underline: false,
            ruler: true,
            grid: false,
            navigation: false,
            zoom: 100.0,
            font: "Calibri".into(),
            font_size: 11.0,
            align: 0,
            log: vec!["Ready".into()],
            frame: 0,
            screenshot_requested: false,
        }
    }
}

impl DemoApp {
    fn ribbon(&mut self, ui: &mut egui::Ui) {
        let events = std::cell::RefCell::new(Vec::<String>::new());
        let click = |r: egui::Response, name: &str| {
            if r.clicked() {
                events.borrow_mut().push(format!("Clicked: {name}"));
            }
        };

        Ribbon::new("main_ribbon").show(ui, |ribbon| {
            ribbon.app_menu("File", |ui| {
                for item in ["New", "Open…", "Save", "Save as…"] {
                    if ui.button(item).clicked() {
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ribbon.tab("Home", |tab| {
                tab.group("Clipboard", |g| {
                    let r = g.large_menu('📋', "Paste", |ui| {
                        let _ = ui.button("Paste");
                        let _ = ui.button("Paste text only");
                        let _ = ui.button("Paste special…");
                    });
                    click(r, "Paste");
                    click(g.small_button('✂', "Cut"), "Cut");
                    click(g.small_button('🗐', "Copy"), "Copy");
                    click(g.small_button('✏', "Format painter"), "Format painter");
                });

                let font = tab.group_with_launcher("Font", |g| {
                    g.add_row(|ui| {
                        egui::ComboBox::from_id_salt("font_family")
                            .width(120.0)
                            .selected_text(&self.font)
                            .show_ui(ui, |ui| {
                                for f in ["Calibri", "Arial", "Times New Roman", "Consolas"] {
                                    ui.selectable_value(&mut self.font, f.to_string(), f);
                                }
                            });
                        ui.add(
                            egui::DragValue::new(&mut self.font_size)
                                .range(6.0..=72.0)
                                .speed(0.5),
                        );
                    });
                    g.row(|r| {
                        let toggles = [
                            (&mut self.bold, 'B', "Bold"),
                            (&mut self.italic, 'I', "Italic"),
                            (&mut self.underline, 'U', "Underline"),
                        ];
                        for (value, ch, hint) in toggles {
                            let resp = r
                                .button(RibbonButton::small("").icon(letter(ch)).selected(*value))
                                .on_hover_text(hint);
                            if resp.clicked() {
                                *value = !*value;
                            }
                        }
                        r.separator();
                        r.small_menu(letter('A').color(Color32::from_rgb(200, 40, 40)), "", |ui| {
                            for name in ["Red", "Green", "Blue"] {
                                let _ = ui.button(name);
                            }
                        });
                    });
                    g.row(|r| {
                        r.icon_button(icon_font('⏶')).on_hover_text("Grow font");
                        r.icon_button(icon_font('⏷')).on_hover_text("Shrink font");
                        r.icon_button(icon_font('🗙')).on_hover_text("Clear formatting");
                    });
                });
                if font.launcher_clicked {
                    events.borrow_mut().push("Font dialog opened".into());
                }

                tab.group("Paragraph", |g| {
                    g.row(|r| {
                        let names = ["Align left", "Center", "Align right"];
                        for (idx, (icon, name)) in ['⬅', '⬌', '➡'].into_iter().zip(names).enumerate() {
                            let resp = r
                                .button(RibbonButton::small("").icon(icon_font(icon)).selected(self.align == idx))
                                .on_hover_text(name);
                            if resp.clicked() {
                                self.align = idx;
                            }
                        }
                    });
                    g.small_menu(icon_font('☰'), "List", |ui| {
                        let _ = ui.button("• Bulleted");
                        let _ = ui.button("1. Numbered");
                    });
                    g.small_button(icon_font('⮫'), "Indent");
                });

                tab.group("Styles", |g| {
                    for (name, color) in [
                        ("Normal", Color32::GRAY),
                        ("Heading 1", Color32::from_rgb(40, 110, 200)),
                        ("Heading 2", Color32::from_rgb(25, 169, 121)),
                    ] {
                        let icon = RibbonIcon::glyph('¶').color(color);
                        click(g.large_button(icon, name), name);
                    }
                });

                tab.group("Editing", |g| {
                    click(g.small_button('🔍', "Find"), "Find");
                    click(g.small_button('🔁', "Replace"), "Replace");
                    g.small_menu('⛶', "Select", |ui| {
                        let _ = ui.button("Select all");
                        let _ = ui.button("Select objects");
                    });
                });

                tab.group("Document", |g| {
                    // SVG icon embedded into the binary.
                    let save = RibbonIcon::image(egui::include_image!("../assets/save.svg"));
                    click(g.large_button(save, "Save"), "Save");
                    let print = RibbonButton::large("Print").icon('🖶').enabled(false);
                    click(g.button(print), "Print");
                });
            });

            ribbon.tab("Insert", |tab| {
                tab.group("Illustrations", |g| {
                    // PNG icons.
                    let picture = RibbonIcon::image(egui::include_image!("../assets/picture.png"));
                    click(g.large_button(picture, "Pictures"), "Pictures");
                    let table = RibbonIcon::image(egui::include_image!("../assets/table.png"));
                    g.large_menu(table, "Table", |ui| {
                        let _ = ui.button("Insert table…");
                        let _ = ui.button("Draw table");
                    });
                    let chart = RibbonIcon::image(egui::include_image!("../assets/chart.svg"));
                    click(g.large_button(chart, "Chart"), "Chart");
                });
                tab.group("Monochrome SVG + tint", |g| {
                    let star = egui::include_image!("../assets/mono_star.svg");
                    for (name, color) in [
                        ("Gold", Color32::from_rgb(230, 170, 20)),
                        ("Red", Color32::from_rgb(210, 60, 60)),
                        ("Blue", Color32::from_rgb(50, 110, 220)),
                    ] {
                        let icon = RibbonIcon::image(star.clone()).color(color);
                        click(g.small_button(icon, name), name);
                    }
                });
                tab.group("Links", |g| {
                    click(g.large_button('🔗', "Link"), "Link");
                    click(g.large_button('🔖', "Bookmark"), "Bookmark");
                    click(g.large_button('💬', "Comment"), "Comment");
                });
                tab.group("Text", |g| {
                    click(g.large_button('🗛', "Text box"), "Text box");
                    click(g.small_button('📅', "Date & time"), "Date & time");
                    click(g.small_button('Ω', "Symbol"), "Symbol");
                    click(g.small_button('∑', "Equation"), "Equation");
                });
            });

            ribbon.tab("View", |tab| {
                tab.group("Show", |g| {
                    g.add(egui::Checkbox::new(&mut self.ruler, "Ruler"));
                    g.add(egui::Checkbox::new(&mut self.grid, "Gridlines"));
                    g.add(egui::Checkbox::new(&mut self.navigation, "Navigation pane"));
                });
                tab.group("Zoom", |g| {
                    if g.large_button('🔍', "Zoom").clicked() {
                        self.zoom = 100.0;
                    }
                    g.add_row(|ui| {
                        ui.add(egui::Slider::new(&mut self.zoom, 10.0..=400.0).suffix("%"))
                    });
                    if g.small_button(icon_font('⟲'), "100%").clicked() {
                        self.zoom = 100.0;
                    }
                });
                tab.group("Window", |g| {
                    g.large_button('🗗', "New window");
                    g.large_button('◫', "Side by side");
                });
            });

            ribbon.tab("Help", |tab| {
                tab.group("Help", |g| {
                    g.large_button('❓', "Help");
                    g.large_button('✉', "Feedback");
                });
            });
        });

        for e in events.into_inner() {
            self.log.push(e);
        }
        if self.log.len() > 8 {
            self.log.drain(..self.log.len() - 8);
        }
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        let Ok(path) = std::env::var("RIBBON_SCREENSHOT") else {
            return;
        };
        ctx.request_repaint();
        if self.frame < 20 {
            return;
        }
        if !self.screenshot_requested {
            self.screenshot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let [w, h] = image.size;
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            image::save_buffer(&path, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
                .expect("save screenshot");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame += 1;
        if let (1, Ok(tab)) = (self.frame, std::env::var("RIBBON_TAB")) {
            Ribbon::select_tab(&ctx, "main_ribbon", &tab);
        }

        egui::Panel::top("ribbon_panel")
            .frame(egui::Frame::NONE.fill(ui.visuals().panel_fill))
            .show(ui, |ui| self.ribbon(ui));

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Document");
            ui.label(format!(
                "Font: {} {} pt, bold: {}, italic: {}, underline: {}, zoom: {:.0}%",
                self.font, self.font_size, self.bold, self.italic, self.underline, self.zoom
            ));
            ui.label("Make the window narrower: scroll arrows appear on the ribbon, and the mouse wheel scrolls it too.");
            ui.label("Double-click a tab to collapse or expand the ribbon.");
            ui.separator();
            for line in &self.log {
                ui.monospace(line);
            }
        });

        self.screenshot(&ctx);
    }
}
