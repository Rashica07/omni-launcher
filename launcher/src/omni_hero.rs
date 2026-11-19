//! OMNI's Home character: the player in 3D, large, wearing their own skin, breathing and
//! looking around as the game's idle player does. Drawn on the CPU by the Wardrobe's renderer
//! (`horizon_cosmetics::preview`) into an image, about 20 times a second while the window is in
//! front; still when it is not, or under reduced motion (`sky::motion`).
//!
//! Which skin: the selected Microsoft account's (from the profile heads' cache), else the offline
//! name's own skin file, else the default skin the game gives that name (from the prepared game
//! files), else a plain figure. Never a stand-in character.
use crate::sky::{motion, Motion};
use gpui::{div, img, prelude::*, px, Context, RenderImage, Window};
use horizon_cosmetics::preview::{self, Camera};
use horizon_cosmetics::{Part, Pose, Texture};
use horizon_launcher::account::Accounts;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The character's box, in points.
pub const W: f32 = 320.;
pub const H: f32 = 470.;
/// One idle frame: the game's tick rate.
const FRAME: Duration = Duration::from_millis(50);
/// The game's ticks per second, which the idle motion is written in.
const TICKS: f32 = 20.;

/// The skin to wear and whether it has slim arms.
#[derive(Clone)]
pub struct Skin {
    pub texture: Arc<Texture>,
    pub slim: bool,
    /// Where it came from, for the line under the name.
    pub source: Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Account,
    OfflineFile,
    Default,
    Plain,
}

impl Source {
    /// What the line under the player says about the skin.
    pub fn label(self) -> &'static str {
        match self {
            Source::Account => "Microsoft account",
            Source::OfflineFile => "Offline · own skin",
            Source::Default => "Offline · default skin",
            Source::Plain => "Skin not available",
        }
    }
}

impl PartialEq for Skin {
    fn eq(&self, other: &Self) -> bool {
        self.texture.rgba == other.texture.rgba && self.slim == other.slim
    }
}

/// The skin the player wears in game, as far as the launcher can know it.
pub fn current_skin(accounts: &Accounts, resources: Option<&Path>, data: &Path) -> Skin {
    if let Some(account) = accounts.selected() {
        if let Some((url, model)) = &account.skin {
            let cache = data.join("profile-heads");
            if let Some(png) = horizon_launcher::heads::cached_skin(url, &cache) {
                if let Some(t) = texture(&png) {
                    return Skin { texture: t, slim: model == "slim", source: Source::Account };
                }
            }
        }
    }
    let name = accounts.name_arg();
    if let Some(name) = name {
        let look = accounts.look(name);
        let (file, _) = horizon_launcher::skins::files(&data.join("skins"), name);
        if look.skin {
            if let Some(t) = std::fs::read(&file).ok().and_then(|b| texture(&b)) {
                let slim = look.model == horizon_launcher::skins::Model::Slim;
                return Skin { texture: t, slim, source: Source::OfflineFile };
            }
        }
        if let Some(skin) = resources.and_then(|r| default_for(name, r)) {
            return skin;
        }
    }
    Skin { texture: Arc::new(plain()), slim: false, source: Source::Plain }
}

/// The default skin the game gives offline `name`, read from prepared game files.
fn default_for(name: &str, resources: &Path) -> Option<Skin> {
    let uuid = horizon_protocol::crypto::offline_uuid(name);
    let which = horizon_launcher::heads::default_skin(uuid);
    let path = resources
        .join("assets/minecraft/textures/entity/player")
        .join(format!("{which}.png"));
    let t = texture(&std::fs::read(path).ok()?)?;
    Some(Skin { texture: t, slim: which.starts_with("slim/"), source: Source::Default })
}

/// A 64×64 skin as the renderer takes it (older 64×32 skins are left to the plain figure).
fn texture(png: &[u8]) -> Option<Arc<Texture>> {
    let image = image::load_from_memory(png).ok()?.to_rgba8();
    (image.dimensions() == (64, 64)).then(|| {
        Arc::new(Texture { width: 64, height: 64, rgba: image.into_raw() })
    })
}

/// A plain figure in three greys, for a player whose skin the launcher cannot read.
fn plain() -> Texture {
    let mut rgba = vec![0u8; 64 * 64 * 4];
    for y in 0..64 {
        for x in 0..64 {
            let c: [u8; 4] = match (x, y) {
                (0..32, 0..16) | (40..56, 16..32) | (32..48, 48..64) => [196, 196, 204, 255],
                (16..40, 16..32) => [128, 132, 148, 255],
                (0..16, 16..32) | (16..32, 48..64) => [92, 94, 108, 255],
                _ => [0, 0, 0, 0],
            };
            rgba[(y * 64 + x) * 4..][..4].copy_from_slice(&c);
        }
    }
    Texture { width: 64, height: 64, rgba }
}

/// The idle player's parts at `ticks`: arms swaying and the head looking around, as small as
/// the game's own (`HumanoidModel.setupAnim` bobbing), plus a slow glance. Pivots are the
/// model's, since a pose replaces the part's whole rest pose.
pub fn idle_pose(ticks: f32, slim: bool) -> Vec<(String, Pose)> {
    let arm_y = if slim { 2.5 } else { 2.0 };
    let sway = (ticks * 0.09).cos() * 0.05 + 0.05;
    let swing = (ticks * 0.067).sin() * 0.05;
    let look_y = (ticks * 0.021).sin() * 0.22;
    let look_x = (ticks * 0.033).sin() * 0.06;
    let p = |x: f32, y: f32, rx: f32, ry: f32, rz: f32| -> Pose { [x, y, 0., rx, ry, rz, 1., 1., 1.] };
    vec![
        ("head".into(), p(0., 0., look_x, look_y, 0.)),
        ("right_arm".into(), p(-5., arm_y, swing, 0., sway)),
        ("left_arm".into(), p(5., arm_y, -swing, 0., -sway)),
    ]
}

/// The camera at `secs`: three-quarters, drifting slowly from side to side.
pub fn camera_at(secs: f32) -> Camera {
    Camera { yaw: 24. + (secs * 0.35).sin() * 12., pitch: 6., zoom: 1.0 }
}

/// A frame from `omni_render` as an image GPUI draws.
fn image(frame: omni_render::Frame) -> Arc<RenderImage> {
    let buffer = image::RgbaImage::from_raw(frame.width, frame.height, frame.bgra)
        .expect("width*height*4 bytes");
    Arc::new(RenderImage::new([image::Frame::new(buffer)]))
}

pub struct Hero {
    skin: Option<Skin>,
    body: Part,
    image: Option<Arc<RenderImage>>,
    replaced: Vec<Arc<RenderImage>>,
    started: Instant,
    /// Seconds already played, while the clock is stopped.
    banked: f32,
    running_since: Option<Instant>,
    drawn_at: Option<f32>,
    waking: bool,
    activation: Option<gpui::Subscription>,
}

impl Hero {
    pub fn new() -> Self {
        Self {
            skin: None,
            body: preview::player_model(false),
            image: None,
            replaced: Vec::new(),
            started: Instant::now(),
            banked: 0.,
            running_since: None,
            drawn_at: None,
            waking: false,
            activation: None,
        }
    }

    /// Where the worn skin came from.
    pub fn source(&self) -> Option<Source> {
        self.skin.as_ref().map(|s| s.source)
    }

    /// Wears `skin`, if it is a new one.
    pub fn wear(&mut self, skin: Skin, cx: &mut Context<Self>) {
        if self.skin.as_ref() == Some(&skin) {
            return;
        }
        self.body = preview::player_model(skin.slim);
        self.skin = Some(skin);
        self.drawn_at = None;
        cx.notify();
    }

    fn secs(&self, now: Instant) -> f32 {
        self.banked + self.running_since.map_or(0., |s| now.duration_since(s).as_secs_f32())
    }

    fn run(&mut self, on: bool, now: Instant) {
        match (on, self.running_since) {
            (true, None) => self.running_since = Some(now),
            (false, Some(_)) => {
                self.banked = self.secs(now);
                self.running_since = None;
            }
            _ => {}
        }
    }

    fn draw(&mut self, secs: f32, scale: f32) {
        let Some(skin) = &self.skin else { return };
        let frame = omni_render::frame(
            &self.body,
            &skin.texture,
            idle_pose(secs * TICKS, skin.slim),
            &camera_at(secs),
            (W as u32, H as u32),
            scale,
        );
        self.replaced.extend(self.image.replace(image(frame)));
    }

    fn wake(&mut self, cx: &mut Context<Self>) {
        if self.waking {
            return;
        }
        self.waking = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FRAME).await;
            let _ = this.update(cx, |this, cx| {
                this.waking = false;
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for Hero {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.started;
        for image in self.replaced.drain(..) {
            let _ = window.drop_image(image);
        }
        if self.activation.is_none() {
            self.activation = Some(cx.observe_window_activation(window, |_, _, cx| cx.notify()));
        }
        let now = Instant::now();
        let moving = match motion() {
            Motion::Still => false,
            Motion::InFront => window.is_window_active(),
            Motion::Always => true,
        };
        self.run(moving, now);
        let secs = self.secs(now);
        if self.drawn_at != Some(secs) {
            self.draw(secs, window.scale_factor());
            self.drawn_at = Some(secs);
        }
        if moving {
            self.wake(cx);
        }
        div()
            .w(px(W))
            .h(px(H))
            .flex_none()
            .children(self.image.clone().map(|i| img(i).w(px(W)).h(px(H))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_idle_motion_stays_small_and_keeps_the_pivots() {
        for slim in [false, true] {
            for t in (0..2000).step_by(7) {
                for (name, pose) in idle_pose(t as f32, slim) {
                    for r in &pose[3..6] {
                        assert!(r.abs() <= 0.25, "{name} turns {r} at {t}");
                    }
                    let pivot = match name.as_str() {
                        "head" => [0., 0.],
                        "right_arm" => [-5., if slim { 2.5 } else { 2. }],
                        "left_arm" => [5., if slim { 2.5 } else { 2. }],
                        other => panic!("{other}"),
                    };
                    assert_eq!([pose[0], pose[1]], pivot, "{name}");
                    assert_eq!(&pose[6..], &[1., 1., 1.]);
                }
            }
        }
    }

    #[test]
    fn the_camera_drifts_within_a_quarter_turn() {
        for s in 0..600 {
            let c = camera_at(s as f32 * 0.1);
            assert!((12. ..=36.).contains(&c.yaw), "{}", c.yaw);
            assert_eq!(c.zoom, 1.0);
        }
    }

    #[test]
    fn the_character_is_drawn_and_fills_its_box() {
        let skin = Arc::new(plain());
        let body = preview::player_model(false);
        let draws = [preview::Draw {
            poses: idle_pose(0., false),
            ..preview::Draw::model(&body, &skin)
        }];
        let picture = preview::render(&draws, W as u32, H as u32, &camera_at(0.));
        let opaque: Vec<usize> = picture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p[3] > 0)
            .map(|(i, _)| i / W as usize)
            .collect();
        assert!(!opaque.is_empty(), "nothing drawn");
        let (top, bottom) = (opaque[0], *opaque.last().unwrap());
        // The figure takes most of the box's height: a character, not an avatar.
        assert!((bottom - top) as f32 > H * 0.6, "{top}..{bottom}");
    }

    #[test]
    fn without_any_skin_source_the_plain_figure_is_worn() {
        let dir = std::env::temp_dir().join(format!("omni-hero-{}", std::process::id()));
        let accounts = Accounts::default();
        let skin = current_skin(&accounts, None, &dir);
        assert_eq!(skin.source, Source::Plain);
    }

    #[test]
    fn an_offline_name_wears_its_default_skin_from_the_game_files() {
        let dir = std::env::temp_dir().join(format!("omni-hero-res-{}", std::process::id()));
        let name = "DevPlayer";
        let which = horizon_launcher::heads::default_skin(
            horizon_protocol::crypto::offline_uuid(name),
        );
        let file = dir
            .join("assets/minecraft/textures/entity/player")
            .join(format!("{which}.png"));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        image::RgbaImage::from_pixel(64, 64, image::Rgba([10, 20, 30, 255]))
            .save(&file)
            .unwrap();
        let accounts = Accounts {
            offline: vec![name.into()],
            offline_selected: Some(name.into()),
            ..Accounts::default()
        };
        let skin = current_skin(&accounts, Some(&dir), &dir);
        assert_eq!(skin.source, Source::Default);
        assert_eq!(skin.slim, which.starts_with("slim/"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod look {
    use super::*;

    /// Writes frames of the character as the dev data folder's player wears it, to look at
    /// (`HORIZON_LAUNCHER_HOME=… cargo test write_frames -- --ignored`).
    #[test]
    #[ignore]
    fn write_frames() {
        let accounts = Accounts::load();
        let tag = horizon_launcher::backend::installed().into_iter().next();
        let resources = tag.map(|t| horizon_launcher::backend::resources(&t));
        let skin = current_skin(&accounts, resources.as_deref(), &horizon_launcher::backend::data_dir());
        println!("skin: {:?} slim {}", skin.source, skin.slim);
        let body = preview::player_model(skin.slim);
        for (i, secs) in [0.0f32, 2.0, 4.5].iter().enumerate() {
            let draws = [preview::Draw {
                poses: idle_pose(secs * TICKS, skin.slim),
                ..preview::Draw::model(&body, &skin.texture)
            }];
            let p = preview::render(&draws, W as u32, H as u32, &camera_at(*secs));
            let out = std::env::temp_dir().join(format!("omni-hero-{i}.png"));
            image::RgbaImage::from_raw(W as u32, H as u32, p.rgba).unwrap().save(&out).unwrap();
            println!("{}", out.display());
        }
    }
}

#[cfg(test)]
mod cost {
    use super::*;

    /// How long one frame takes to draw (`cargo test frame_cost -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn frame_cost() {
        let skin = Arc::new(plain());
        let body = preview::player_model(false);
        let start = Instant::now();
        let mut render = Duration::ZERO;
        for i in 0..40 {
            let draws = [preview::Draw {
                poses: idle_pose(i as f32, false),
                ..preview::Draw::model(&body, &skin)
            }];
            let r = Instant::now();
            let p = preview::render(&draws, W as u32, H as u32, &camera_at(i as f32 * 0.05));
            render += r.elapsed();
            let _ = image(omni_render::scale_up(&p.rgba, W as u32, H as u32, 2.));
        }
        println!("ms per frame: {:.2}", start.elapsed().as_secs_f64() * 1000. / 40.);
        println!("of which render: {:.2}", render.as_secs_f64() * 1000. / 40.);
    }
}
