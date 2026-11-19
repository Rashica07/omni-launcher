// OMNI's opening film, the clock: `omni_intro_engine::tick` adds the frame's time once the film has
// been shown and hands back the next film frame (once per film frame), which goes up as the mod image.
fn omni_intro_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    if let Some(rgba) = omni_intro_engine::tick(elapsed.as_secs_f64()) {
        let _ = game.native_mod_image(omni_intro_engine::WIDTH, omni_intro_engine::HEIGHT, &rgba);
    }
}
