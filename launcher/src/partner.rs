//! The partner slot: ASTRO Launcher, a Minecraft launcher that partners with OMNI. Its wide banner
//! on the Servers page, and a text card for the sidebar on the other pages. The banner picture is
//! in the binary, never fetched; a click opens the partner's page in the browser. A "Partner" label
//! above each says what it is. After `SEEN_BEFORE_HIDE` seconds on screen a "Hide" appears on
//! either, and hiding one hides both for good.
use crate::theme::Theme;
use gpui::{div, img, prelude::*, px, rgb, ClickEvent, Div, FontWeight, Image, ImageFormat, Stateful};
use std::sync::{Arc, LazyLock};

/// The partner's page. ASTRO Launcher has no site of its own in our records yet, so this opens its
/// maintainer's GitHub; change it here once the project has a home page.
pub const URL: &str = "https://github.com/jojq21";

/// How long the sidebar's banner stays on screen, counted over every start, before a "Hide" shows
/// beside its label: long enough to be seen, then the player may put it away for good.
pub const SEEN_BEFORE_HIDE: u64 = 180;

/// The small "Hide" beside a banner's or card's label, top right of whatever holds it (which must be
/// `relative`). The caller adds the click: it sets `Settings::partner_hidden`, which hides every
/// partner banner for good.
pub fn hide_button(id: &'static str, t: &Theme) -> Stateful<Div> {
    let t = *t;
    let hide = div()
        .id(id)
        .absolute()
        .top(px(0.))
        .right(px(0.))
        .cursor_pointer()
        .child(
            crate::text(crate::theme::MICRO)
                .text_color(t.muted)
                .hover(move |s| s.text_color(t.ink))
                .child(horizon_launcher::i18n::tr("HIDE")),
        );
    crate::keyboard(hide, &t)
}

/// 1200×441: twice the widest it is shown at, for Retina screens.
const WIDTH: f32 = 1200.;
const HEIGHT: f32 = 441.;
static BANNER: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        ImageFormat::Png,
        include_bytes!("../assets/astro-banner.png").to_vec(),
    ))
});

/// The banner `width` logical pixels wide, under its label. Hovering lifts it a little and turns the
/// label to ink. The narrowest window (960) leaves the page 648 pixels, so the Servers one always fits.
pub fn banner(id: &'static str, width: f32, t: &Theme) -> Stateful<Div> {
    let t = *t;
    let el = div()
        .id(id)
        .group(id)
        .flex()
        .flex_col()
        .gap(px(8.))
        .w(px(width))
        .cursor_pointer()
        .child(
            crate::text(crate::theme::MICRO)
                .text_color(t.muted)
                .group_hover(id, move |s| s.text_color(t.ink))
                .child(horizon_launcher::i18n::tr("PARTNER")),
        )
        .child(
            div().relative().group_hover(id, |s| s.top(px(-2.))).child(
                img(BANNER.clone())
                    .w(px(width))
                    .h(px(width * HEIGHT / WIDTH)),
            ),
        )
        .on_click(|_: &ClickEvent, _, cx| cx.open_url(URL));
    crate::keyboard(el, &t)
}

/// The banner alone, `width` logical pixels wide, for a card that says "Partner" itself (the
/// glass Servers page). Hovering lifts it a little.
pub fn banner_image(id: &'static str, width: f32, t: &Theme) -> Stateful<Div> {
    let el = div()
        .id(id)
        .group(id)
        .w(px(width))
        .cursor_pointer()
        .child(
            div().relative().group_hover(id, |s| s.top(px(-2.))).child(
                img(BANNER.clone())
                    .w(px(width))
                    .h(px(width * HEIGHT / WIDTH)),
            ),
        )
        .on_click(|_: &ClickEvent, _, cx| cx.open_url(URL));
    crate::keyboard(el, t)
}

/// OMNI's colours: the card is the same in both themes, as a partner's card is.
const CARD: u32 = 0x1a1a1d;
const RED: u32 = 0xc83246;
const RED_HOVER: u32 = 0xdc4a5e;
const CREAM: u32 = 0xe8dcc8;
const GOLD: u32 = 0xe8b44f;
const GREY: u32 = 0xa39a8b;

/// What the tall card needs of the sidebar's height, label included: below it the sidebar shows
/// the compact card.
pub const TALL_HEIGHT: f32 = 250.;

/// The partner's card for the sidebar, `width` logical pixels wide: its name, one line about the
/// partnership and a button, all drawn by the launcher so they stay sharp and follow its
/// language. `compact` drops the button, for windows too short for the tall card. Hovering lifts
/// it a little.
pub fn card(id: &'static str, width: f32, compact: bool, t: &Theme) -> Stateful<Div> {
    let t = *t;
    let label = crate::text(crate::theme::MICRO)
        .text_color(t.muted)
        .group_hover(id, move |s| s.text_color(t.ink))
        .child(horizon_launcher::i18n::tr("PARTNER"));
    let radius = if t.glass { 10. } else { 0. };
    let name = div()
        .flex()
        .flex_col()
        .child(
            div()
                .text_size(px(if compact { 18. } else { 24. }))
                .line_height(px(if compact { 22. } else { 28. }))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CREAM))
                .child("ASTRO"),
        )
        .child(
            div()
                .text_size(px(11.))
                .line_height(px(16.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(GOLD))
                .child("LAUNCHER"),
        );
    let mut inner = div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .p(px(if compact { 12. } else { 16. }))
        .child(div().w(px(32.)).h(px(3.)).bg(rgb(RED)))
        .child(name);
    if !compact {
        inner = inner
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(rgb(GREY))
                    .child("Partner of OMNI"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(34.))
                    .rounded(px(if t.glass { 8. } else { 0. }))
                    .bg(rgb(RED))
                    .group_hover(id, |s| s.bg(rgb(RED_HOVER)))
                    .text_color(rgb(0xffffff))
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Visit ASTRO Launcher"),
            );
    }
    let face = div()
        .relative()
        .flex()
        .flex_col()
        .flex_none()
        .w(px(width))
        .bg(rgb(CARD))
        .rounded(px(radius))
        .group_hover(id, |s| s.top(px(-2.)))
        .child(inner);
    let el = div()
        .id(id)
        .group(id)
        .flex()
        .flex_col()
        .gap(px(8.))
        .w(px(width))
        .cursor_pointer()
        .child(label)
        .child(face)
        .on_click(|_: &ClickEvent, _, cx| cx.open_url(URL));
    crate::keyboard(el, &t)
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, WIDTH};

    #[test]
    fn the_bundled_banner_decodes() {
        let bytes = include_bytes!("../assets/astro-banner.png");
        let banner = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .expect("the banner is a PNG");
        assert_eq!(
            (banner.width(), banner.height()),
            (WIDTH as u32, HEIGHT as u32)
        );
    }

    #[test]
    fn the_partner_link_is_https() {
        assert!(super::URL.starts_with("https://"));
    }
}
