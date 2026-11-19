//! OMNI's opening film for OMNI Client: the cubes assembling into the OMNI logo (rendered in
//! Blender, `intro/omni_intro.py`; our own scene, nothing from Mojang). No game types here: the
//! bundle's helpers (`../frame.rs`, `../post.rs`) upload the frame and draw it.
//!
//! Time only runs once the film has been *shown* (a frame was asked for), so a slow start-up
//! (resource loading hides the GUI pass) never eats the film.
use std::sync::Mutex;

/// The film: `count` PNG frames of `WIDTH` x `HEIGHT`, each length-prefixed (little endian u32).
static FRAMES: &[u8] = include_bytes!("../data/frames.bin");
pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 270;
pub const FPS: f64 = 24.0;
/// After the last frame the charcoal fades into the title for this long.
pub const FADE: f64 = 0.6;
/// The page colour of the film's background (#1A1A1D), as ARGB.
pub const CHARCOAL: u32 = 0xFF1A1A1D;

/// What to draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// Which film frame to show; `None` while the film is over and only the fade is left.
    pub frame: Option<usize>,
    /// Opacity of the charcoal under (and after) the film, 0..=1.
    pub cover: f32,
}

struct State {
    t: f64,
    shown: bool,
    uploaded: Option<usize>,
}
static STATE: Mutex<State> = Mutex::new(State { t: 0.0, shown: false, uploaded: None });

/// How many frames the film has.
pub fn len() -> usize {
    FRAMES.get(..4).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
}

/// Seconds the film plays, fade not included.
pub fn length() -> f64 {
    len() as f64 / FPS
}

/// The view at `t` seconds after the film was first shown; `None` once it is all over.
pub fn view_at(t: f64) -> Option<View> {
    let end = length();
    if t < end {
        let last = len().saturating_sub(1);
        Some(View { frame: Some(((t * FPS) as usize).min(last)), cover: 1.0 })
    } else if t < end + FADE {
        Some(View { frame: None, cover: (1.0 - (t - end) / FADE) as f32 })
    } else {
        None
    }
}

/// Called by the GUI pass: marks the film as shown (the clock starts) and says what to draw.
pub fn view() -> Option<View> {
    let mut s = STATE.lock().ok()?;
    s.shown = true;
    view_at(s.t)
}

/// Called once a frame by the `frame` hook: advances the clock if the film is showing, and returns
/// the RGBA of the frame that has to be uploaded (once per film frame), if any.
pub fn tick(elapsed: f64) -> Option<Vec<u8>> {
    let mut s = STATE.lock().ok()?;
    if s.shown {
        s.t += elapsed.clamp(0.0, 0.25);
    }
    let want = view_at(s.t)?.frame?;
    if s.uploaded == Some(want) {
        return None;
    }
    s.uploaded = Some(want);
    decode(want)
}

/// The RGBA pixels of frame `index` (`WIDTH * HEIGHT * 4` bytes).
pub fn decode(index: usize) -> Option<Vec<u8>> {
    let mut at = 4;
    for i in 0..len() {
        let n = u32::from_le_bytes(FRAMES.get(at..at + 4)?.try_into().ok()?) as usize;
        at += 4;
        if i == index {
            let mut dec = png::Decoder::new(std::io::Cursor::new(FRAMES.get(at..at + n)?));
            dec.set_transformations(png::Transformations::EXPAND);
            let mut r = dec.read_info().ok()?;
            let mut buf = vec![0; r.output_buffer_size()?];
            let info = r.next_frame(&mut buf).ok()?;
            if (info.width, info.height) != (WIDTH, HEIGHT) {
                return None;
            }
            buf.truncate(info.buffer_size());
            return match info.color_type {
                png::ColorType::Rgba => Some(buf),
                png::ColorType::Rgb => Some(buf.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect()),
                _ => None,
            };
        }
        at += n;
    }
    None
}

/// Where the film goes in a window `w` x `h` GUI pixels: as large as fits, centred.
pub fn contain(w: f32, h: f32) -> [f32; 4] {
    let s = (w / WIDTH as f32).min(h / HEIGHT as f32);
    let (dw, dh) = (WIDTH as f32 * s, HEIGHT as f32 * s);
    [(w - dw) / 2.0, (h - dh) / 2.0, dw, dh]
}

/// The cover's ARGB at opacity `cover`.
pub fn cover_argb(cover: f32) -> u32 {
    let a = (cover.clamp(0.0, 1.0) * 255.0).round() as u32;
    (a << 24) | (CHARCOAL & 0x00FF_FFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_film_is_84_frames_three_and_a_half_seconds() {
        assert_eq!(len(), 84);
        assert!((length() - 3.5).abs() < 1e-9);
    }
    #[test]
    fn views_follow_the_clock_then_fade_then_end() {
        assert_eq!(view_at(0.0), Some(View { frame: Some(0), cover: 1.0 }));
        assert_eq!(view_at(1.0).unwrap().frame, Some(24));
        assert_eq!(view_at(3.49).unwrap().frame, Some(83));
        let mid = view_at(3.5 + FADE / 2.0).unwrap();
        assert_eq!(mid.frame, None);
        assert!((mid.cover - 0.5).abs() < 1e-3);
        assert_eq!(view_at(3.5 + FADE + 0.01), None);
    }
    #[test]
    fn every_frame_decodes_to_the_right_size() {
        for i in 0..len() {
            assert_eq!(decode(i).expect("frame").len(), (WIDTH * HEIGHT * 4) as usize, "frame {i}");
        }
        assert!(decode(len()).is_none());
    }
    #[test]
    fn the_film_fits_the_window_without_cropping() {
        let [x, y, w, h] = contain(1920.0, 1080.0);
        assert_eq!((x, y, w, h), (0.0, 0.0, 1920.0, 1080.0));
        let [x, y, w, h] = contain(1000.0, 1000.0);
        assert!((w - 1000.0).abs() < 1e-3 && (h - 562.5).abs() < 1e-3 && (y - 218.75).abs() < 1e-3 && x.abs() < 1e-3);
    }
    #[test]
    fn the_cover_colour_is_the_page_charcoal() {
        assert_eq!(cover_argb(1.0), 0xFF1A1A1D);
        assert_eq!(cover_argb(0.0), 0x001A1A1D);
    }
}
