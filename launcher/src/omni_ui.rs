//! OMNI's design primitives. When the OMNI theme is on (and not the glass look), the shared
//! builders in `blocks`, `shell`, `plate` and `main` (`button`, `slot_row`) ask [`on`] and draw
//! OMNI's version: soft 8 pt panels with one hairline instead of bevelled boxes, underlined tabs,
//! hairline-divided rows instead of one card per row, and the interface typeface instead of the
//! pixel font. One switch, so every page changes together.
use gpui::{rgb, Hsla};
use std::cell::Cell;

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
}

/// Set once per frame by the window (`Launcher::render`).
pub fn set_on(on: bool) {
    ON.with(|c| c.set(on));
}

pub fn on() -> bool {
    ON.with(Cell::get)
}

/// Corner radii: panels and plates, controls.
pub const RADIUS: f32 = 8.;
pub const RADIUS_CONTROL: f32 = 6.;

/// The surface a panel is filled with: the brand charcoal, lifted, nearly opaque over the scene.
pub fn surface() -> Hsla {
    Hsla::from(rgb(0x232327)).opacity(0.88)
}

/// A raised surface: a selected or hovered row, a secondary button.
pub fn raised() -> Hsla {
    Hsla::from(rgb(0x2C2C31))
}

/// Hairlines: cream at 8 %.
pub fn hairline() -> Hsla {
    Hsla::from(rgb(0xE8DCC8)).opacity(0.08)
}

/// The brand red, for the one thing that matters on a surface (the open tab's bar).
pub fn red() -> Hsla {
    Hsla::from(rgb(0xC83246))
}

/// The red a filled button wears: cream text on it reaches 4.78:1.
pub fn red_fill() -> Hsla {
    Hsla::from(rgb(0xB02A3C))
}

/// A text field in OMNI's look: one hairline (red while focused), soft corners, a dark well.
pub fn soft_field<E: gpui::Styled>(e: E, focused: bool) -> E {
    e.border_1()
        .rounded(gpui::px(RADIUS))
        .border_color(if focused { red() } else { hairline() })
        .bg(rgb(0x1A1A1D).opacity(0.6))
}

/// "INSTANCES" → "Instances": a heading written in capitals, in sentence case; text already in
/// mixed case is left as written.
pub fn sentence_case(text: &str) -> String {
    if text.chars().any(char::is_lowercase) {
        return text.to_string();
    }
    let lower = text.to_lowercase();
    let mut chars = lower.chars();
    chars
        .next()
        .map(|f| f.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitals_become_sentence_case_and_mixed_case_stays() {
        assert_eq!(sentence_case("INSTANCES"), "Instances");
        assert_eq!(sentence_case("MY SERVERS"), "My servers");
        assert_eq!(sentence_case("Add a native mod from GitHub"), "Add a native mod from GitHub");
        assert_eq!(sentence_case(""), "");
    }

    fn contrast(a: Hsla, b: Hsla) -> f32 {
        fn lum(c: Hsla) -> f32 {
            let c = gpui::Rgba::from(c);
            let f = |v: f32| if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
            0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
        }
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn cream_reads_on_every_surface_and_on_the_red_fill() {
        let cream = Hsla::from(rgb(0xE8DCC8));
        let muted = Hsla::from(rgb(0x9A9387));
        for bg in [surface().opacity(1.), raised()] {
            assert!(contrast(cream, bg) >= 4.5);
            assert!(contrast(muted, bg) >= 4.5);
        }
        assert!(contrast(cream, red_fill()) >= 4.5);
    }

    #[test]
    fn the_switch_is_per_thread_and_starts_off() {
        assert!(!on());
        set_on(true);
        assert!(on());
        set_on(false);
        assert!(!on());
    }
}
