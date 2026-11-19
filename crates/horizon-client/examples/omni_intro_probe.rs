//! Real-renderer check of OMNI Client's opening film (`native-mods/omni-intro`): the same engine and
//! the same drawing calls as the bundle's helpers, through the public trait API.
//! `cargo run -p horizon-client --example omni_intro_probe -- --resources PATH`, with a
//! `native-mods.json` (`{"mods":[{"id":"omni_intro"}]}`) beside the executable: the game only makes the
//! mod-image slot when it was built with native mods. `OMNI_INTRO_SLOW=10` plays the film 10x slower.
use horizon_client::mod_api::{FrameEvent, GuiRenderPostEvent, Hooks, Host};

#[path = "../../../native-mods/omni-intro/engine/src/lib.rs"]
mod engine;

struct Intro;

impl Hooks for Intro {
    fn frame(&self, host: &mut dyn Host, event: &mut FrameEvent) {
        if let Some(rgba) = engine::tick(event.elapsed.as_secs_f64() / std::env::var("OMNI_INTRO_SLOW").ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0)) {
            let _ = host.mod_image(engine::WIDTH, engine::HEIGHT, &rgba);
        }
    }

    fn gui_render_post(&self, host: &dyn Host, event: &mut GuiRenderPostEvent<'_>) {
        let Some(view) = engine::view() else { return };
        let (w, h) = (event.batch.width as f32, event.batch.height as f32);
        host.fill(event.batch, [0.0, 0.0, w, h], engine::cover_argb(view.cover));
        if view.frame.is_some() {
            let src = [0.0, 0.0, engine::WIDTH as f32, engine::HEIGHT as f32];
            host.sprite(event.batch, "horizon/modimage", src, engine::contain(w, h));
        }
    }
}

fn main() {
    horizon_client::entry::main_with(vec![Box::new(Intro) as Box<dyn Hooks>].into());
}
