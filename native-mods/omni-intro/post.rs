// OMNI's opening film, the picture: charcoal over the whole window, the frame uploaded by
// `frame.rs` as large as fits, then only the charcoal, fading out into the title screen.
fn omni_intro_post(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    let Some(view) = omni_intro_engine::view() else {
        return;
    };
    let (w, h) = (batch.width as f32, batch.height as f32);
    game.native_fill(batch, [0.0, 0.0, w, h], omni_intro_engine::cover_argb(view.cover));
    if view.frame.is_some() {
        let src = [0.0, 0.0, omni_intro_engine::WIDTH as f32, omni_intro_engine::HEIGHT as f32];
        game.native_sprite(batch, "horizon/modimage", src, omni_intro_engine::contain(w, h));
    }
}
