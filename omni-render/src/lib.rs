//! The Home character's frame: the player model drawn by `horizon_cosmetics::preview`, then
//! turned into what GPUI draws (BGRA, premultiplied alpha, each pixel a whole block of device
//! pixels). Its own crate so `[profile.dev.package.omni-render]` can optimise it: drawn 20 times a
//! second, it would cost ~30 ms a frame unoptimised.
use horizon_cosmetics::preview::{self, Camera};
use horizon_cosmetics::{Part, Pose, Texture};

/// A drawn frame: `width` × `height` device pixels, BGRA, alpha premultiplied.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Draws `body` wearing `skin`, posed by `poses`, in a `w` × `h` point box seen by `camera`, at
/// `scale` device pixels a point.
pub fn frame(
    body: &Part,
    skin: &Texture,
    poses: Vec<(String, Pose)>,
    camera: &Camera,
    (w, h): (u32, u32),
    scale: f32,
) -> Frame {
    let draws = [preview::Draw { poses, ..preview::Draw::model(body, skin) }];
    let picture = preview::render(&draws, w, h, camera);
    scale_up(&picture.rgba, w, h, scale)
}

/// `rgba` (`w` × `h`) as GPUI's BGRA with premultiplied alpha, each pixel repeated to cover
/// whole device pixels at `scale`.
pub fn scale_up(rgba: &[u8], w: u32, h: u32, scale: f32) -> Frame {
    let dw = ((w as f32 * scale).round() as u32).max(1);
    let dh = ((h as f32 * scale).round() as u32).max(1);
    let (w, h, dwu, dhu) = (w as usize, h as usize, dw as usize, dh as usize);
    // One converted source row, then each device row copies its source row's pixels.
    let xs: Vec<usize> = (0..dwu).map(|x| (x * w / dwu).min(w - 1)).collect();
    let mut bgra = vec![0u8; dwu * dhu * 4];
    let mut row = vec![0u8; w * 4];
    let mut last = usize::MAX;
    for y in 0..dhu {
        let sy = (y * h / dhu).min(h - 1);
        if sy != last {
            for x in 0..w {
                let i = (sy * w + x) * 4;
                let (r, g, b, a) = (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]);
                let pre = |c: u8| (c as u16 * a as u16 / 255) as u8;
                row[x * 4..x * 4 + 4].copy_from_slice(&[pre(b), pre(g), pre(r), a]);
            }
            last = sy;
        }
        let out = &mut bgra[y * dwu * 4..(y + 1) * dwu * 4];
        for (dx, &sx) in xs.iter().enumerate() {
            out[dx * 4..dx * 4 + 4].copy_from_slice(&row[sx * 4..sx * 4 + 4]);
        }
    }
    Frame { width: dw, height: dh, bgra }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubling_repeats_each_pixel_and_premultiplies() {
        // Two pixels: opaque red, half-transparent white.
        let rgba = [255, 0, 0, 255, 255, 255, 255, 128];
        let f = scale_up(&rgba, 2, 1, 2.);
        assert_eq!((f.width, f.height), (4, 2));
        let px = |i: usize| &f.bgra[i * 4..i * 4 + 4];
        assert_eq!(px(0), [0, 0, 255, 255]);
        assert_eq!(px(1), [0, 0, 255, 255]);
        assert_eq!(px(2), [128, 128, 128, 128]);
        // The second row is the first again.
        assert_eq!(&f.bgra[16..32], &f.bgra[0..16]);
    }

    #[test]
    fn scale_one_keeps_the_size() {
        let f = scale_up(&[1, 2, 3, 255], 1, 1, 1.);
        assert_eq!((f.width, f.height, f.bgra.clone()), (1, 1, vec![3, 2, 1, 255]));
    }
}
