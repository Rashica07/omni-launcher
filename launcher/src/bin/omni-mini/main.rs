//! OMNI Launcher: its own interface, on Horizon's launcher engine (`horizon_launcher`).
mod import;
mod palette;
mod play;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, ClickEvent, Context, FontWeight, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use horizon_launcher::instances::{self, Instance};
use palette as c;
use std::borrow::Cow;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Instances,
    Import,
}

impl Page {
    const ALL: [(Page, &'static str); 3] =
        [(Page::Home, "Home"), (Page::Instances, "Instances"), (Page::Import, "Import")];
}

#[derive(Clone, PartialEq)]
enum State {
    Idle,
    Starting(Vec<String>),
    Running,
    Failed(String),
}

struct App_ {
    page: Page,
    instances: Vec<Instance>,
    selected: String,
    state: State,
    tx: Sender<play::Event>,
    rx: Receiver<play::Event>,
    found: Option<Vec<horizon_launcher::other_launchers::Found>>,
    importing: Option<String>,
    import_note: Option<String>,
    import_tx: Sender<Result<Instance, String>>,
    import_rx: Receiver<Result<Instance, String>>,
}

impl App_ {
    fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = channel();
        let (import_tx, import_rx) = channel();
        let instances = instances::list();
        let selected = instances::selected().id;
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_millis(100)).await;
            if this.update(cx, |this, cx| this.drain(cx)).is_err() {
                break;
            }
        })
        .detach();
        Self { page: Page::Home, instances, selected, state: State::Idle, tx, rx, found: None, importing: None, import_note: None, import_tx, import_rx }
    }

    fn current(&self) -> Option<&Instance> {
        self.instances.iter().find(|i| i.id == self.selected).or(self.instances.first())
    }

    fn busy(&self) -> bool {
        matches!(self.state, State::Starting(_) | State::Running)
    }

    fn drain(&mut self, cx: &mut Context<Self>) {
        let mut changed = false;
        while let Ok(done) = self.import_rx.try_recv() {
            changed = true;
            self.importing = None;
            match done {
                Ok(instance) => {
                    self.import_note = Some(format!("Imported {}.", instance.name));
                    self.instances = instances::list();
                }
                Err(e) => self.import_note = Some(e),
            }
        }
        while let Ok(event) = self.rx.try_recv() {
            changed = true;
            self.state = match (event, std::mem::replace(&mut self.state, State::Idle)) {
                (play::Event::Line(l), State::Starting(mut lines)) => {
                    lines.push(l);
                    let keep = lines.len().saturating_sub(4);
                    lines.drain(..keep);
                    State::Starting(lines)
                }
                (play::Event::Line(_), s) => s,
                (play::Event::Started, _) => State::Running,
                (play::Event::Finished(Ok(())), _) => State::Idle,
                (play::Event::Finished(Err(e)), _) => State::Failed(e),
            };
        }
        if changed {
            cx.notify();
        }
    }

    fn play(&mut self, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        if let Some(instance) = self.current().cloned() {
            self.state = State::Starting(vec![]);
            play::start(instance, self.tx.clone());
            cx.notify();
        }
    }

    fn scan(&mut self, cx: &mut Context<Self>) {
        self.found = Some(import::scan());
        self.import_note = None;
        cx.notify();
    }

    fn import_one(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.importing.is_some() {
            return;
        }
        if let Some(found) = self.found.as_ref().and_then(|f| f.get(index)).cloned() {
            self.importing = Some(found.name.clone());
            self.import_note = None;
            import::start(found, self.import_tx.clone());
            cx.notify();
        }
    }

    fn import_page(&self, cx: &mut Context<Self>) -> gpui::Div {
        let list = self.found.clone();
        let button = |id: &'static str, label: &'static str| {
            div().id(id).h(px(40.)).px(px(16.)).flex().items_center().rounded(px(10.))
                .cursor_pointer().bg(rgb(c::RAISED)).hover(|s| s.opacity(0.85)).child(label)
        };
        div()
            .flex().flex_col().gap(px(12.))
            .child(div().text_size(px(28.)).font_weight(FontWeight::SEMIBOLD).child("Import"))
            .child(div().text_color(rgb(c::MUTED)).child(
                "Finds Prism, MultiMC, CurseForge, Modrinth, ATLauncher and the official launcher on this computer. Your originals are copied, never changed.",
            ))
            .child(button("scan", "Find instances").on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.scan(cx))))
            .children(self.importing.as_ref().map(|n| div().text_color(rgb(c::GOLD)).child(SharedString::from(format!("Copying {n}…")))))
            .children(self.import_note.as_ref().map(|n| div().text_color(rgb(c::GOLD)).child(SharedString::from(n.clone()))))
            .children(match &list {
                Some(l) if l.is_empty() => Some(div().text_color(rgb(c::MUTED)).child("Nothing found in the usual places.")),
                _ => None,
            })
            .children(list.unwrap_or_default().into_iter().enumerate().map(|(i, f)| {
                div().id(SharedString::from(format!("found{i}")))
                    .h(px(60.)).px(px(16.)).flex().items_center().justify_between()
                    .rounded(px(14.)).bg(rgb(c::SURFACE))
                    .child(div().flex().flex_col()
                        .child(SharedString::from(f.name.clone()))
                        .child(div().text_size(px(12.)).text_color(rgb(c::MUTED)).child(SharedString::from(import::describe(&f)))))
                    .child(div().id(SharedString::from(format!("imp{i}"))).h(px(36.)).px(px(14.)).flex().items_center()
                        .rounded(px(10.)).cursor_pointer().bg(rgb(c::RED_FILL)).child("Import")
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.import_one(i, cx))))
            }))
    }

    fn select(&mut self, id: String, cx: &mut Context<Self>) {
        if instances::select(&id).is_ok() {
            self.selected = id;
        }
        cx.notify();
    }

    fn tab(&self, page: Page, label: &'static str, cx: &mut Context<Self>) -> impl IntoElement {
        let on = self.page == page;
        div()
            .id(label)
            .h(px(40.))
            .px(px(16.))
            .flex()
            .items_center()
            .rounded(px(10.))
            .cursor_pointer()
            .text_color(rgb(if on { c::CREAM } else { c::MUTED }))
            .bg(rgb(if on { c::RAISED } else { c::SURFACE }))
            .hover(|s| s.bg(rgb(c::RAISED)))
            .child(label)
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.page = page;
                cx.notify();
            }))
    }

    fn home(&self, cx: &mut Context<Self>) -> gpui::Div {
        let Some(instance) = self.current() else {
            return div().child("No instances yet.");
        };
        let refused = play::playable(instance).err();
        let (label, enabled) = match &self.state {
            State::Starting(_) => ("Starting…", false),
            State::Running => ("Playing", false),
            _ => ("Play", refused.is_none()),
        };
        let detail: SharedString = match (&self.state, refused) {
            (State::Starting(lines), _) => lines.last().cloned().unwrap_or_default().into(),
            (State::Failed(e), _) => e.clone().into(),
            (_, Some(why)) => why.into(),
            _ => instance
                .java
                .as_ref()
                .map(|g| format!("Minecraft {}", g.minecraft))
                .unwrap_or_default()
                .into(),
        };
        let failed = matches!(self.state, State::Failed(_));
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(div().text_size(px(13.)).text_color(rgb(c::MUTED)).child("Ready to play"))
            .child(
                div()
                    .text_size(px(40.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(SharedString::from(instance.name.clone())),
            )
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(if failed { c::GOLD } else { c::MUTED }))
                    .child(detail),
            )
            .child(
                div()
                    .id("play")
                    .w(px(200.))
                    .h(px(52.))
                    .rounded(px(14.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(c::CREAM))
                    .bg(rgb(if enabled { c::RED_FILL } else { c::RAISED }))
                    .when(enabled, |b| b.cursor_pointer().hover(|s| s.opacity(0.9)))
                    .child(label)
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.play(cx))),
            )
    }

    fn instances_page(&self, cx: &mut Context<Self>) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(div().text_size(px(28.)).font_weight(FontWeight::SEMIBOLD).child("Instances"))
            .children(self.instances.iter().map(|i| {
                let on = i.id == self.selected;
                let id = i.id.clone();
                let kind = match &i.java {
                    Some(g) => format!("Minecraft {}", g.minecraft),
                    None => "OMNI Client".to_string(),
                };
                div()
                    .id(SharedString::from(i.id.clone()))
                    .h(px(60.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(px(14.))
                    .cursor_pointer()
                    .bg(rgb(if on { c::RAISED } else { c::SURFACE }))
                    .border_1()
                    .border_color(rgb(if on { c::RED } else { c::SURFACE }))
                    .child(SharedString::from(i.name.clone()))
                    .child(div().text_color(rgb(c::MUTED)).child(SharedString::from(kind)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.select(id.clone(), cx)
                    }))
            }))
    }
}

impl Render for App_ {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs: Vec<_> = Page::ALL.iter().map(|(p, l)| self.tab(*p, l, cx).into_any_element()).collect();
        let page = match self.page {
            Page::Home => self.home(cx),
            Page::Instances => self.instances_page(cx),
            Page::Import => self.import_page(cx),
        };
        div()
            .size_full()
            .flex()
            .bg(rgb(c::CHARCOAL))
            .text_color(rgb(c::CREAM))
            .font_family("Geist")
            .child(
                div()
                    .w(px(220.))
                    .h_full()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .p(px(16.))
                    .pt(px(44.))
                    .bg(rgb(c::SURFACE))
                    .child(
                        div()
                            .px(px(16.))
                            .pb(px(16.))
                            .text_size(px(18.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("OMNILauncher"),
                    )
                    .children(tabs),
            )
            .child(div().flex_1().p(px(40.)).pt(px(56.)).child(page))
    }
}

fn main() {
    let _ = horizon_launcher::data_location::startup();
    horizon_launcher::proxy::announce(horizon_launcher::backend::Settings::load().proxy);
    gpui_platform::application().run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(horizon_theme::fonts::GEIST_REGULAR),
                Cow::Borrowed(horizon_theme::fonts::GEIST_MEDIUM),
                Cow::Borrowed(horizon_theme::fonts::GEIST_SEMIBOLD),
            ])
            .expect("bundled fonts");
        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.), px(560.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("OMNILauncher".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                app_id: Some("omni-launcher".into()),
                ..Default::default()
            },
            |_, cx| cx.new(App_::new),
        )
        .expect("window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
