//! An MS Office-style ribbon for [egui](https://github.com/emilk/egui).
//!
//! * tabs, captioned groups with a dialog launcher button;
//! * large and small buttons, toggles, dropdown menu buttons;
//! * small items are automatically stacked into columns of three;
//! * any egui widget inside groups ([`GroupUi::add`], [`GroupUi::add_row`]);
//! * icons from fonts or PNG/SVG images ([`RibbonIcon`]);
//! * horizontal scrolling when the groups don't fit the width;
//! * collapsing the ribbon by double-clicking a tab.
//!
//! ```no_run
//! use ribbon_egui::{Ribbon, RibbonIcon};
//!
//! fn ui(ui: &mut egui::Ui, bold: &mut bool) {
//!     Ribbon::new("main_ribbon").show(ui, |ribbon| {
//!         ribbon.tab("Home", |tab| {
//!             tab.group("Clipboard", |g| {
//!                 g.large_button('📋', "Paste");
//!                 g.small_button('✂', "Cut");
//!                 g.small_button(RibbonIcon::uri("file://icons/copy.svg"), "Copy");
//!             });
//!             tab.group("Font", |g| {
//!                 g.small_toggle(bold, 'B', "Bold");
//!             });
//!         });
//!     });
//! }
//! ```

mod button;
mod group;
mod icon;
mod ribbon;
mod style;

pub use button::{ButtonSize, RibbonButton};
pub use group::{GroupResponse, GroupUi, TabUi};
pub use icon::RibbonIcon;
pub use ribbon::{Ribbon, RibbonUi};
pub use style::RibbonStyle;
