# OMNI Launcher: audit, redesign plan and roadmap

Written 2026-11-18. The brief calls the base an "Astra fork". It is a fork of **Horizon Launcher** (Rust, GPUI, MIT, see `launcher/NOTICE-OMNI.md`). ASTRO/Astra is only the partner banner and the screenshots, which are reference only.

## 1. Technical audit

- **Stack:** Rust. GPUI (`gpui-fast` 0.1.13) draws the window natively; there is no web view and no CSS. Every visual change is Rust code.
- **Shape:** `launcher/src/lib.rs` is the engine (instances, accounts, Mojang downloads, Java, loaders, mods, imports). `launcher/src/main.rs` holds one `Launcher` struct (~6,000 lines) with the pages as `impl Launcher` files (`home.rs`, `instances_page.rs`, `mods_page.rs`, ...).
- **Two looks already exist:** "glass" (rounded translucent plates) and "Blocks" (square, solid). Themes with their own palette, like OMNI, draw Blocks. Shared primitives decide most of the look: `plate.rs`, `shell.rs`, `blocks.rs`, `theme.rs`, `themes.rs`, `glass_ui.rs`.
- **3D:** `horizon_cosmetics::preview` is a CPU rasteriser for the player model and skins, used by the Wardrobe (`wardrobe_page/viewer.rs`). No GPU 3D library is needed.
- **Motion:** `sky::motion()` follows the OS's reduced-motion setting. Animations stop when the window is unfocused.
- **Tests:** 649 lib + 117 bin + integration. They run after every change.

## 2. Functionality to preserve (nothing below may break)

Version install and update · preparing resources from an own install or Mojang · instances (create, edit, select, delete, icons, groups) · Java instances with loaders (Fabric, Forge, NeoForge, Quilt) · import from Prism, MultiMC, CurseForge, Modrinth, ATLauncher, official launcher · modpacks (Modrinth .mrpack, Horizon registry) · mod browser, install, remove, auto-update · worlds, resource and shader packs, screenshots · servers list and dedicated server · Microsoft sign-in and the owner gate; offline names (dev-only offline play without an owner) · skins and capes for offline names, the Wardrobe · settings (folders, memory, proxy, language, accent, background) · CLI and MCP · OMNI Client (Horizon Rust client) launch and crash reports.

## 3. Redesign proposal

- **Feeling:** a game you enter, not a dashboard. The backdrop is the blurred voxel sunset under a dark veil. Pages read like editorial spreads: one dominant thing per screen, a strong type scale, cream on charcoal, red only for the one action that matters, gold for status.
- **Home (centrepiece):** the player in 3D, large, on the right, wearing their own skin, with idle breathing, an arm sway and a slow glance, and the camera drifting a little. On the left: instance name in display type, Minecraft version and loader, readiness, and one big Play button. Under the player: name and where the skin comes from. Patch notes and friends move below the fold.
- **Navigation:** a narrow vertical rail of icon and label pairs fixed to the left edge, part of the backdrop rather than a grey sidebar. The active item gets a red notch and cream text. Order: Home, Instances, Modpacks, Mods, Servers, Downloads, Settings; the rest (Worlds, Skins, Wardrobe, Screenshots) sits under the instance or player.
- **Avoid:** cards everywhere, big radii, gradients, glass on everything, fake stats, placeholder buttons.

## 4. Component and animation plan

- **Type scale:** Display 44/48 semibold, Title 28, Heading 18, Body 15/1.5, Caption 13, Mono 13 for paths and versions.
- **Spacing:** a 4 pt grid; gaps of 8, 12, 16, 24, 32, 48.
- **Radius:** 4 for controls and 8 for panels. Square where Minecraft is (slots, icons).
- **Surfaces:** charcoal page; raised `#232327`; plate `#2C2C31`; hairline borders at cream 8 %.
- **Buttons:** Primary (red fill `#B02A3C`, cream text 4.78:1), Secondary (plate), Ghost (text). Each has hover, press (scale 0.97, 100 ms), focus ring, disabled and loading states.
- **Motion:** 120 ms for hover and press, 220 ms for page and panel changes, ease-out-quint, no bounce on UI chrome. Under reduced motion, cross-fades only and the character stands still.

## 5. Roadmap (stages, each tested before the next)

1. **Home character (done in this pass):** `src/omni_hero.rs`, wired into Home when the OMNI theme is on. Skin source: Microsoft account, then the offline name's own skin, then the game's default skin for that name, then a plain figure. 5 tests.
2. **Home layout (done):** display-type instance name, one Play button, status line; remove the leftover Horizon banner parts in the OMNI look.
3. **Navigation rail (done; Servers moved to OMNIServers):** rework `sidebar()` and the tab order; give it the active notch and hover timing.
4. **Design primitives (done):** OMNI variants of `plate`, `shell::row`, `button` and `notice`, so every page changes together.
5. **Instances, Mods, Modpacks, Downloads, Settings (done):** soft search field, source switch, sort dropdown, text fields (`omni_ui::soft_field`), and an OMNI opening (mark, name, red line). Also rounded: Home Play, Version picker, invite field, Instances icon tiles, memory slider. Still square: a few console/agent fields.
6. **Interaction polish (done, partly unverified):** page transitions, instance selection animation, toasts, loading states.
7. **Leftovers:** telemetry and update sources still point at Horizon; "Horizon" still appears in logs and folder names; then a release build check and cross-platform tests.
