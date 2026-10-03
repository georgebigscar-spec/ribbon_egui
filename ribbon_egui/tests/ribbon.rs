use egui::{vec2, Event, Modifiers, MouseWheelUnit, TouchPhase};
use egui_kittest::{kittest::Queryable, Harness};
use ribbon_egui::Ribbon;

fn two_tabs(ui: &mut egui::Ui) {
    Ribbon::new("test").show(ui, |r| {
        r.tab("Home", |t| {
            t.group("Clipboard", |g| {
                g.large_button('📋', "Paste");
            });
        });
        r.tab("Insert", |t| {
            t.group("Illustrations", |g| {
                g.large_button('🖼', "Pictures");
            });
        });
    });
}

#[test]
fn clicking_tab_switches_content() {
    let mut harness = Harness::builder().with_size(vec2(600.0, 200.0)).build_ui(two_tabs);
    harness.run();
    assert!(harness.query_by_label("Paste").is_some());
    assert!(harness.query_by_label("Pictures").is_none());

    harness.get_by_label("Insert").click();
    harness.run();
    assert!(harness.query_by_label("Paste").is_none());
    assert!(harness.query_by_label("Pictures").is_some());
}

#[test]
fn double_click_collapses_ribbon() {
    // Short time step so both clicks fit into the double-click interval.
    let mut harness = Harness::builder()
        .with_size(vec2(600.0, 200.0))
        .with_step_dt(0.02)
        .build_ui(two_tabs);
    harness.run();
    harness.get_by_label("Home").click();
    harness.step();
    harness.get_by_label("Home").click();
    harness.run();
    assert!(harness.query_by_label("Paste").is_none());

    harness.get_by_label("Home").click();
    harness.run();
    assert!(harness.query_by_label("Paste").is_some());
}

#[test]
fn small_buttons_stack_in_columns_of_three() {
    let mut harness = Harness::builder()
        .with_size(vec2(600.0, 200.0))
        .build_ui(|ui| {
            Ribbon::new("test").show(ui, |r| {
                r.tab("Home", |t| {
                    t.group("Group", |g| {
                        for name in ["A1", "A2", "A3", "B1"] {
                            g.small_button('•', name);
                        }
                    });
                });
            });
        });
    harness.run();
    let a1 = harness.get_by_label("A1").rect();
    let a2 = harness.get_by_label("A2").rect();
    let b1 = harness.get_by_label("B1").rect();
    assert_eq!(a1.left(), a2.left());
    assert!(a2.top() > a1.top());
    assert!(b1.left() > a1.right());
    assert_eq!(b1.top(), a1.top());
}

#[test]
fn mouse_wheel_scrolls_overflowing_ribbon() {
    let width = 300.0;
    let mut harness = Harness::builder()
        .with_size(vec2(width, 200.0))
        .build_ui(|ui| {
            Ribbon::new("test").show(ui, |r| {
                r.tab("Home", |t| {
                    for i in 0..6 {
                        t.group(&format!("Group {i}"), |g| {
                            g.large_button('★', &format!("Button {i}"));
                            g.large_button('★', &format!("More {i}"));
                        });
                    }
                });
            });
        });
    harness.run();
    let before = harness.get_by_label("More 5").rect();
    assert!(before.right() > width, "last button should start off-screen");

    // Hover the ribbon and scroll the wheel down: the ribbon moves left.
    let pos = harness.get_by_label("Button 0").rect().center();
    for _ in 0..20 {
        harness.event(Event::PointerMoved(pos));
        harness.event(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: vec2(0.0, -200.0),
            phase: TouchPhase::Move,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }
    harness.run();
    let after = harness.get_by_label("More 5").rect();
    assert!(after.left() < before.left());
    assert!(after.right() <= width, "last button should be scrolled into view: {after:?}");
}

#[test]
fn toggle_flips_value() {
    let mut bold = false;
    {
        let mut harness = Harness::builder()
            .with_size(vec2(600.0, 200.0))
            .build_ui(|ui| {
                Ribbon::new("test").show(ui, |r| {
                    r.tab("Home", |t| {
                        t.group("Font", |g| {
                            g.small_toggle(&mut bold, 'B', "Bold");
                        });
                    });
                });
            });
        harness.run();
        harness.get_by_label("Bold").click();
        harness.run();
    }
    assert!(bold);
}
