//! The OMNI brand colours (assets/brand/tokens.json): one place, so the UI never uses a raw hex.

pub const CHARCOAL: u32 = 0x1A1A1D;
pub const SURFACE: u32 = 0x232327;
pub const RAISED: u32 = 0x2C2C31;
pub const CREAM: u32 = 0xE8DCC8;
pub const MUTED: u32 = 0x9A9387;
pub const RED: u32 = 0xC83246;
/// Filled buttons: the brand red, deepened until cream text on it reaches 4.5:1.
pub const RED_FILL: u32 = 0xB02A3C;
pub const GOLD: u32 = 0xE8B44F;

/// WCAG contrast ratio between two 0xRRGGBB colours (used by the readability tests).
#[cfg(test)]
pub fn contrast(a: u32, b: u32) -> f64 {
    fn lum(c: u32) -> f64 {
        let f = |v: u32| {
            let s = v as f64 / 255.;
            if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f((c >> 16) & 255) + 0.7152 * f((c >> 8) & 255) + 0.0722 * f(c & 255)
    }
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_colours_are_readable_on_their_surfaces() {
        for bg in [CHARCOAL, SURFACE, RAISED] {
            assert!(contrast(CREAM, bg) >= 4.5, "cream on {bg:06x}");
            assert!(contrast(MUTED, bg) >= 4.5, "muted on {bg:06x}");
        }
        // Cream text on the red Play button.
        assert!(contrast(CREAM, RED_FILL) >= 4.5, "cream on red");
    }
}
