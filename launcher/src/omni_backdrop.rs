//! OMNI's backdrop: a voxel sunset rendered in Blender (`OMNI/assets/background/build_scene.py`),
//! shipped as three WebP layers (sky and far land, clouds, near land) and animated here with slow
//! parallax: the layers drift against each other on sine curves, so the loop never jumps.
//!
//! It is cheap on purpose. Nothing is decoded or drawn per frame beyond moving three images; a
//! frame is asked for about 15 times a second only while the window is in front, and never under
//! reduced motion. It follows the launcher's own motion rules (`sky::motion`): it stops 10 s after
//! the window loses focus and picks up where it stopped.
use crate::sky::{motion, Clock, Motion};
use gpui::{div, img, prelude::*, px, Div, Image, ImageFormat};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

/// The renders' size: 16:9.
const ASPECT: f32 = 16. / 9.;
/// How long the backdrop keeps moving after the window loses focus.
const UNFOCUSED: Duration = Duration::from_secs(10);
/// A frame about 15 times a second: slow drift needs no more.
/// How much of the picture the dark veil covers.
const SCRIM: f32 = 0.8;
const FRAME: Duration = Duration::from_millis(66);
/// The layers are drawn this much larger than the window, so their drift never shows an edge.
const OVERSCAN: f32 = 1.06;

fn layer(bytes: &'static [u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Webp, bytes.to_vec()))
}
static SKY: LazyLock<Arc<Image>> = LazyLock::new(|| layer(include_bytes!("../assets/omni-bg.webp")));
static CLOUDS: LazyLock<Arc<Image>> =
    LazyLock::new(|| layer(include_bytes!("../assets/omni-clouds.webp")));
static NEAR: LazyLock<Arc<Image>> = LazyLock::new(|| layer(include_bytes!("../assets/omni-fg.webp")));

/// How far each layer drifts, in logical pixels either side of centre, and how long one turn
/// takes. The near land moves the most and against the sky, as in a real parallax.
const SKY_DRIFT: (f32, f32) = (6., 70.);
const CLOUD_DRIFT: (f32, f32) = (46., 110.);
const NEAR_DRIFT: (f32, f32) = (-16., 70.);

fn drift((amount, period): (f32, f32), t: f32, phase: f32) -> f32 {
    amount * (std::f32::consts::TAU * t / period + phase).sin()
}

/// How far a layer is offset at time `t` seconds, for the three layers: (sky, clouds, near).
pub fn offsets(t: f32) -> (f32, f32, f32) {
    (
        drift(SKY_DRIFT, t, 0.),
        drift(CLOUD_DRIFT, t, 1.3),
        drift(NEAR_DRIFT, t, 0.),
    )
}

/// The size and top-left of a layer that covers a `w` by `h` window, with room to drift.
pub fn cover(w: f32, h: f32) -> (f32, f32, f32, f32) {
    let width = w.max(h * ASPECT) * OVERSCAN;
    let height = width / ASPECT;
    (width, height, (w - width) / 2., (h - height) / 2.)
}

#[derive(Default)]
pub struct OmniBackdrop {
    clock: Clock,
    unfocused: Option<Instant>,
}

impl OmniBackdrop {
    /// The backdrop for a window of `w` by `h` logical pixels, and how long until it next changes
    /// (`None`: it stands still until something else redraws the window).
    pub fn render(
        &mut self,
        (w, h): (f32, f32),
        focused: bool,
        now: Instant,
    ) -> (Div, Option<Duration>) {
        if focused {
            self.unfocused = None;
        } else {
            self.unfocused.get_or_insert(now);
        }
        let run = match motion() {
            Motion::Still => false,
            Motion::InFront => self.unfocused.is_none_or(|since| now - since < UNFOCUSED),
            Motion::Always => true,
        };
        self.clock.set(run, now);
        let t = self.clock.at(now).as_secs_f32();
        let (sky, clouds, near) = offsets(t);
        let (width, height, left, top) = cover(w, h);
        let place = |image: &Arc<Image>, dx: f32| {
            img(image.clone())
                .absolute()
                .left(px(left + dx))
                .top(px(top))
                .w(px(width))
                .h(px(height))
        };
        let scene = div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .overflow_hidden()
            .child(place(&SKY, sky))
            .child(place(&CLOUDS, clouds))
            .child(place(&NEAR, near))
            // A dark veil over the picture, so cream text on top stays readable (WCAG 4.5:1).
            .child(div().absolute().top_0().left_0().size_full().bg(gpui::hsla(0., 0., 0.04, SCRIM)));
        (scene, run.then_some(FRAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layers_decode_at_the_render_size() {
        for bytes in [
            &include_bytes!("../assets/omni-bg.webp")[..],
            &include_bytes!("../assets/omni-clouds.webp")[..],
            &include_bytes!("../assets/omni-fg.webp")[..],
        ] {
            let pic = image::load_from_memory_with_format(bytes, image::ImageFormat::WebP)
                .expect("a WebP");
            assert_eq!((pic.width(), pic.height()), (1600, 900));
        }
    }

    /// Cream text (0xE8DCC8) over even a pure white pixel of the picture, under the veil.
    #[test]
    fn cream_text_stays_readable_over_the_brightest_possible_pixel() {
        let lin = |v: f32| {
            let s = v / 255.;
            if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        let lum = |r: f32, g: f32, b: f32| 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
        let veil = 0.04 * 255.;
        let under = SCRIM * veil + (1. - SCRIM) * 255.;
        let ratio = (lum(232., 220., 200.) + 0.05) / (lum(under, under, under) + 0.05);
        assert!(ratio >= 4.5, "{ratio}");
    }

    #[test]
    fn the_sky_covers_every_window_shape_even_at_full_drift() {
        // only the sky is opaque: the clouds and the near land may drift past the edge
        for (w, h) in [(960., 640.), (1440., 900.), (2560., 1100.), (700., 1200.), (3440., 1440.)] {
            let (width, height, left, top) = cover(w, h);
            assert!(width >= w && height >= h, "{w}x{h}");
            assert!(-left >= SKY_DRIFT.0.abs(), "{w}x{h}: the sky's side edge would show");
            assert!(top <= 0., "{w}x{h}");
        }
    }

    #[test]
    fn the_motion_loops_without_a_jump() {
        // each layer's offset is a sine of its own period: the end of its turn is its start
        for (drift_of, (_, period)) in [(0usize, SKY_DRIFT), (1, CLOUD_DRIFT), (2, NEAR_DRIFT)] {
            let at = |t: f32| match drift_of {
                0 => offsets(t).0,
                1 => offsets(t).1,
                _ => offsets(t).2,
            };
            assert!((at(0.) - at(period)).abs() < 1e-3);
        }
    }

    #[test]
    fn the_offsets_stay_inside_their_amounts() {
        for i in 0..400 {
            let (a, b, c) = offsets(i as f32 * 0.7);
            assert!(a.abs() <= SKY_DRIFT.0.abs() + 1e-3);
            assert!(b.abs() <= CLOUD_DRIFT.0.abs() + 1e-3);
            assert!(c.abs() <= NEAR_DRIFT.0.abs() + 1e-3);
        }
    }
}
