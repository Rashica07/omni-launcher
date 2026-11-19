//! OMNI's opening film: 84 frames (3.5 s at 24 fps) rendered in Blender from `intro/omni_intro.py`,
//! our own scene (flat voxel cubes that fly in and lock into the OMNI ring). Nothing from Mojang.
//! The frames are JPEGs in the binary; each is decoded when first drawn. Under reduced motion the
//! last frame stands.
use gpui::{Image, ImageFormat};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

pub const FPS: f32 = 24.;

const BYTES: [&[u8]; 84] = [
    include_bytes!("../assets/omni-intro/000.jpg"),
    include_bytes!("../assets/omni-intro/001.jpg"),
    include_bytes!("../assets/omni-intro/002.jpg"),
    include_bytes!("../assets/omni-intro/003.jpg"),
    include_bytes!("../assets/omni-intro/004.jpg"),
    include_bytes!("../assets/omni-intro/005.jpg"),
    include_bytes!("../assets/omni-intro/006.jpg"),
    include_bytes!("../assets/omni-intro/007.jpg"),
    include_bytes!("../assets/omni-intro/008.jpg"),
    include_bytes!("../assets/omni-intro/009.jpg"),
    include_bytes!("../assets/omni-intro/010.jpg"),
    include_bytes!("../assets/omni-intro/011.jpg"),
    include_bytes!("../assets/omni-intro/012.jpg"),
    include_bytes!("../assets/omni-intro/013.jpg"),
    include_bytes!("../assets/omni-intro/014.jpg"),
    include_bytes!("../assets/omni-intro/015.jpg"),
    include_bytes!("../assets/omni-intro/016.jpg"),
    include_bytes!("../assets/omni-intro/017.jpg"),
    include_bytes!("../assets/omni-intro/018.jpg"),
    include_bytes!("../assets/omni-intro/019.jpg"),
    include_bytes!("../assets/omni-intro/020.jpg"),
    include_bytes!("../assets/omni-intro/021.jpg"),
    include_bytes!("../assets/omni-intro/022.jpg"),
    include_bytes!("../assets/omni-intro/023.jpg"),
    include_bytes!("../assets/omni-intro/024.jpg"),
    include_bytes!("../assets/omni-intro/025.jpg"),
    include_bytes!("../assets/omni-intro/026.jpg"),
    include_bytes!("../assets/omni-intro/027.jpg"),
    include_bytes!("../assets/omni-intro/028.jpg"),
    include_bytes!("../assets/omni-intro/029.jpg"),
    include_bytes!("../assets/omni-intro/030.jpg"),
    include_bytes!("../assets/omni-intro/031.jpg"),
    include_bytes!("../assets/omni-intro/032.jpg"),
    include_bytes!("../assets/omni-intro/033.jpg"),
    include_bytes!("../assets/omni-intro/034.jpg"),
    include_bytes!("../assets/omni-intro/035.jpg"),
    include_bytes!("../assets/omni-intro/036.jpg"),
    include_bytes!("../assets/omni-intro/037.jpg"),
    include_bytes!("../assets/omni-intro/038.jpg"),
    include_bytes!("../assets/omni-intro/039.jpg"),
    include_bytes!("../assets/omni-intro/040.jpg"),
    include_bytes!("../assets/omni-intro/041.jpg"),
    include_bytes!("../assets/omni-intro/042.jpg"),
    include_bytes!("../assets/omni-intro/043.jpg"),
    include_bytes!("../assets/omni-intro/044.jpg"),
    include_bytes!("../assets/omni-intro/045.jpg"),
    include_bytes!("../assets/omni-intro/046.jpg"),
    include_bytes!("../assets/omni-intro/047.jpg"),
    include_bytes!("../assets/omni-intro/048.jpg"),
    include_bytes!("../assets/omni-intro/049.jpg"),
    include_bytes!("../assets/omni-intro/050.jpg"),
    include_bytes!("../assets/omni-intro/051.jpg"),
    include_bytes!("../assets/omni-intro/052.jpg"),
    include_bytes!("../assets/omni-intro/053.jpg"),
    include_bytes!("../assets/omni-intro/054.jpg"),
    include_bytes!("../assets/omni-intro/055.jpg"),
    include_bytes!("../assets/omni-intro/056.jpg"),
    include_bytes!("../assets/omni-intro/057.jpg"),
    include_bytes!("../assets/omni-intro/058.jpg"),
    include_bytes!("../assets/omni-intro/059.jpg"),
    include_bytes!("../assets/omni-intro/060.jpg"),
    include_bytes!("../assets/omni-intro/061.jpg"),
    include_bytes!("../assets/omni-intro/062.jpg"),
    include_bytes!("../assets/omni-intro/063.jpg"),
    include_bytes!("../assets/omni-intro/064.jpg"),
    include_bytes!("../assets/omni-intro/065.jpg"),
    include_bytes!("../assets/omni-intro/066.jpg"),
    include_bytes!("../assets/omni-intro/067.jpg"),
    include_bytes!("../assets/omni-intro/068.jpg"),
    include_bytes!("../assets/omni-intro/069.jpg"),
    include_bytes!("../assets/omni-intro/070.jpg"),
    include_bytes!("../assets/omni-intro/071.jpg"),
    include_bytes!("../assets/omni-intro/072.jpg"),
    include_bytes!("../assets/omni-intro/073.jpg"),
    include_bytes!("../assets/omni-intro/074.jpg"),
    include_bytes!("../assets/omni-intro/075.jpg"),
    include_bytes!("../assets/omni-intro/076.jpg"),
    include_bytes!("../assets/omni-intro/077.jpg"),
    include_bytes!("../assets/omni-intro/078.jpg"),
    include_bytes!("../assets/omni-intro/079.jpg"),
    include_bytes!("../assets/omni-intro/080.jpg"),
    include_bytes!("../assets/omni-intro/081.jpg"),
    include_bytes!("../assets/omni-intro/082.jpg"),
    include_bytes!("../assets/omni-intro/083.jpg"),
];

pub const LEN: usize = BYTES.len();

/// How long the film runs: the opening holds at least this long in the OMNI look.
pub fn length() -> Duration {
    Duration::from_secs_f32(LEN as f32 / FPS)
}

/// The frame to draw `secs` into the film; the last one once it ends, or under reduced motion.
pub fn frame_index(secs: f32, still: bool) -> usize {
    if still {
        return LEN - 1;
    }
    ((secs.max(0.) * FPS) as usize).min(LEN - 1)
}

static FRAMES: LazyLock<Vec<Arc<Image>>> = LazyLock::new(|| {
    BYTES
        .iter()
        .map(|b| Arc::new(Image::from_bytes(ImageFormat::Jpeg, b.to_vec())))
        .collect()
});

pub fn frame(index: usize) -> Arc<Image> {
    FRAMES[index.min(LEN - 1)].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_film_is_three_and_a_half_seconds() {
        assert_eq!(LEN, 84);
        assert!((length().as_secs_f32() - 3.5).abs() < 0.01);
    }

    #[test]
    fn frames_follow_the_clock_and_stand_on_the_last_one() {
        assert_eq!(frame_index(0., false), 0);
        assert_eq!(frame_index(1., false), 24);
        assert_eq!(frame_index(99., false), LEN - 1);
        assert_eq!(frame_index(-1., false), 0);
        assert_eq!(frame_index(0., true), LEN - 1);
    }

    #[test]
    fn every_frame_is_a_jpeg() {
        for (i, b) in BYTES.iter().enumerate() {
            assert_eq!(&b[..2], &[0xFF, 0xD8], "frame {i}");
        }
    }
}
