//! The Join popup: OMNIServers' Play opens the launcher (`horizon_launcher::join`) and it asks
//! which instance to play the server on. On the left, the server, the instances that play its
//! Minecraft version (or a new one) and whether to remember the choice for this server; on the
//! right, an invite for friends (English only for now, like the other OMNI additions); Cancel and Launch at the foot. Launch has a sheen that sweeps
//! across it while the pointer is on it.
use super::*;
use crate::glass_instance::{list_box, list_choice, section, tick};
use crate::glass_ui::{self as ui, white};
use gpui::relative;
use horizon_launcher::instances::Instance;
use horizon_launcher::join;
use horizon_theme::glass::{ink, radius};
use std::time::Instant;

/// What the popup is asking, and what has been picked so far.
pub(super) struct JoinDraft {
    request: join::Request,
    /// The instances that play the server's version, the saved one first.
    options: Vec<Instance>,
    /// An index into `options`; `options.len()` is a new instance.
    picked: usize,
    remember: bool,
    error: Option<String>,
}

impl JoinDraft {
    /// The draft for `request`: the instance saved for the server (else the first that fits,
    /// else a new one) picked, remembering on.
    pub(super) fn new(request: join::Request, all: &[Instance], data: &std::path::Path) -> Self {
        let saved = join::saved(data, &request.address);
        let options = join::candidates(all, &request.minecraft, saved.as_deref());
        JoinDraft {
            request,
            options,
            picked: 0,
            remember: true,
            error: None,
        }
    }

    fn pick(&self) -> join::Pick {
        match self.options.get(self.picked) {
            Some(i) => join::Pick::Existing(Box::new(i.clone())),
            None => join::Pick::New,
        }
    }
}

/// How often the launcher looks for a request OMNIServers wrote.
const POLL: std::time::Duration = std::time::Duration::from_millis(1000);

impl Launcher {
    /// Called from the launcher's tick: opens the popup when OMNIServers asked for one. True when
    /// the window should be drawn again.
    pub(super) fn poll_join(&mut self) -> bool {
        if self.join_polled.elapsed() < POLL {
            return false;
        }
        self.join_polled = Instant::now();
        let data = backend::data_dir();
        if !join::waiting(&data) {
            return false;
        }
        let Some(request) = join::take(&data, join::now()) else {
            return false;
        };
        self.reload_instances();
        self.join = Some(JoinDraft::new(request, &self.instances.list, &data));
        true
    }

    /// Launch: makes or takes the instance, puts the server in its list, selects it and plays
    /// straight into the server.
    fn launch_join(&mut self) {
        let Some(draft) = self.join.as_mut() else {
            return;
        };
        let data = backend::data_dir();
        let made = join::prepare(&draft.request, draft.pick(), draft.remember, &data);
        match made {
            Ok(instance) => {
                let address = draft.request.address.clone();
                self.join = None;
                self.reload_instances();
                let chosen = self
                    .instances
                    .list
                    .iter()
                    .find(|i| i.id == instance.id)
                    .cloned()
                    .unwrap_or(instance);
                self.select_instance(&chosen);
                self.quick_play("--server", address);
            }
            Err(e) => draft.error = Some(e),
        }
    }

    pub(super) fn join_dialog(&mut self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        self.join.as_ref()?;
        let main = self.join_main(cx);
        let aside = self.join_aside();
        let close = cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.join = None;
            cx.notify();
        });
        let card = ui::dialog_plate(radius::MODAL)
            .id("join-card")
            .occlude()
            .flex()
            .w_full()
            .max_w(px(980.))
            .max_h(px(620.))
            .child(main)
            .child(aside);
        Some(
            div()
                .id("join-dialog")
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(24.))
                .occlude()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .child(ui::scrim_backdrop())
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(ui::scrim())
                        .on_mouse_down(gpui::MouseButton::Left, close),
                )
                .child(card),
        )
    }

    /// The left column: which server, which instance, remember, Cancel and Launch.
    fn join_main(&mut self, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let Some(d) = self.join.as_ref() else {
            return div().id("join-main");
        };
        let soft = ui::look().accent_soft;
        let (name, version, address) = (
            d.request.name.trim().to_string(),
            d.request.minecraft.trim().to_string(),
            d.request.address.trim().to_string(),
        );
        let (picked, remember, error) = (d.picked, d.remember, d.error.clone());
        let mut rows = div().flex().flex_col().gap(px(2.)).p(px(4.));
        for (i, instance) in d.options.iter().enumerate() {
            rows = rows.child(
                list_choice(format!("join-opt-{i}"), picked == i, &t)
                    .h(px(52.))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(d) = this.join.as_mut() {
                            d.picked = i;
                        }
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                ui::text(15., FontWeight::SEMIBOLD, ink::FULL)
                                    .truncate()
                                    .child(instance.name.clone()),
                            )
                            .child(
                                ui::text(13., FontWeight::NORMAL, ink::NOTE)
                                    .child(format!("Minecraft {version}")),
                            ),
                    )
                    .child(tick(picked == i, soft)),
            );
        }
        let new_at = d.options.len();
        rows = rows.child(
            list_choice("join-opt-new", picked == new_at, &t)
                .h(px(52.))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(d) = this.join.as_mut() {
                        d.picked = new_at;
                    }
                    cx.notify();
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            ui::text(15., FontWeight::SEMIBOLD, ink::FULL)
                                .child("New instance"),
                        )
                        .child(
                            ui::text(13., FontWeight::NORMAL, ink::NOTE)
                                .child(format!("Made for Minecraft {version}")),
                        ),
                )
                .child(tick(picked == new_at, soft)),
        );

        let remember_row = list_choice("join-remember", false, &t)
            .h(px(44.))
            .on_click(cx.listener(|this, _, _, cx| {
                if let Some(d) = this.join.as_mut() {
                    d.remember = !d.remember;
                }
                cx.notify();
            }))
            .child(tick(remember, soft))
            .child(
                ui::text(14., FontWeight::MEDIUM, ink::FULL)
                    .child(format!("Save this instance for {name}")),
            );

        let cancel = ui::button("join-cancel".into(), "Cancel".into(), Kind::Secondary, &t)
            .on_click(cx.listener(|this, _, _, cx| {
                this.join = None;
                cx.notify();
            }));
        let launch = self
            .join_launch_button()
            .on_click(cx.listener(|this, _, _, cx| {
                this.launch_join();
                cx.notify();
            }));

        div()
            .id("join-main")
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(20.))
            .p(px(32.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        ui::text(26., FontWeight::SEMIBOLD, ink::FULL)
                            .child(format!("Play {name}")),
                    )
                    .child(
                        ui::text(14., FontWeight::NORMAL, ink::NOTE)
                            .child(format!("Minecraft {version} · {address}")),
                    ),
            )
            .child(section(
                "Instance".into(),
                None,
                list_box("join-list", 280.).child(rows),
            ))
            .child(remember_row)
            .children(error.map(|e| {
                ui::text(13., FontWeight::NORMAL, ink::BODY)
                    .text_color(ui::danger())
                    .child(e)
            }))
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(12.))
                    .child(cancel)
                    .child(launch),
            )
    }

    /// Launch: the accent button; while hovered a bright band sweeps across it.
    fn join_launch_button(&self) -> Stateful<Div> {
        let t = self.theme;
        let look = ui::look();
        let id: SharedString = "join-launch".into();
        let sheen = hover_fade(
            &id,
            || {
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .overflow_hidden()
                    .rounded(px(radius::CONTROL))
            },
            |d, p| {
                // The band is brightest mid-sweep and gone at both ends.
                let glow = 1. - (2. * p - 1.).powi(2);
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .h_full()
                        .w(relative(0.35))
                        .left(relative(-0.35 + 1.35 * p))
                        .bg(white(0.32 * glow)),
                )
            },
        );
        let button = div()
            .id(id.clone())
            .relative()
            .overflow_hidden()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(44.))
            .px(px(28.))
            .rounded(px(radius::CONTROL))
            .cursor_pointer()
            .bg(ui::color(look.accent))
            .text_color(ui::color(look.on_accent))
            .font_family(horizon_theme::glass::FONT)
            .text_size(px(15.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(sheen)
            .child(div().relative().child("Launch"));
        keyboard(track_hover(button, id), &t)
    }

    /// The right column: invite friends.
    fn join_aside(&mut self) -> Stateful<Div> {
        let t = self.theme;
        let Some(d) = self.join.as_ref() else {
            return div().id("join-aside");
        };
        let address = d.request.invite_address().to_string();
        let public = d.request.has_public_address();
        let text = d.request.invite_text();
        let note = if public {
            "Friends anywhere can join with this address."
        } else {
            "No public address yet: this one works on your network only."
        };
        let copy_address = ui::button(
            "join-copy-address".into(),
            "Copy address".into(),
            Kind::Secondary,
            &t,
        )
        .on_click({
            let address = address.clone();
            move |_, _, cx| copy_to_clipboard(cx, gpui::ClipboardItem::new_string(address.clone()))
        });
        let copy_invite = ui::button(
            "join-copy-invite".into(),
            "Copy invite message".into(),
            Kind::Secondary,
            &t,
        )
        .on_click(move |_, _, cx| {
            copy_to_clipboard(cx, gpui::ClipboardItem::new_string(text.clone()))
        });
        div()
            .id("join-aside")
            .flex()
            .flex_col()
            .flex_none()
            .w(px(300.))
            .gap(px(14.))
            .p(px(28.))
            .bg(white(0.04))
            .child(ui::text(18., FontWeight::SEMIBOLD, ink::FULL).child("Invite friends"))
            .child(ui::text(13., FontWeight::NORMAL, ink::NOTE).child(note))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(44.))
                    .px(px(14.))
                    .rounded(px(radius::CONTROL))
                    .bg(white(0.07))
                    .child(
                        ui::text(14., FontWeight::MEDIUM, ink::FULL)
                            .truncate()
                            .child(address),
                    ),
            )
            .child(copy_address)
            .child(copy_invite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> join::Request {
        join::Request {
            name: "Survival".into(),
            address: "localhost:25565".into(),
            public_address: None,
            minecraft: "1.21.4".into(),
            created: 0,
        }
    }

    #[test]
    fn with_no_instance_the_draft_picks_a_new_one_and_remembers() {
        let d = JoinDraft::new(request(), &[], std::path::Path::new("/nonexistent"));
        assert!(d.options.is_empty());
        assert!(matches!(d.pick(), join::Pick::New));
        assert!(d.remember);
        assert!(d.error.is_none());
    }
}
