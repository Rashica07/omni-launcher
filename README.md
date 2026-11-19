# OMNI Launcher

A Minecraft launcher with its own look, built on the MIT-licensed [Horizon](https://github.com/horizon-rust/horizon) launcher engine (Rust + GPUI). Not an official Minecraft product; not approved by or associated with Mojang or Microsoft.

- Source of the launcher: [`launcher/`](launcher/) (see `launcher/NOTICE-OMNI.md` for what changed from Horizon).
- Dev builds for macOS (Apple Silicon), Windows and Linux: [Releases](../../releases). They are unsigned.
- Brand and logo files: [`brand/`](brand/); the intro film's Blender script: [`intro/`](intro/).
- Licence: MIT, with the original Horizon copyright kept in `LICENSE`. Third-party notices: `THIRD_PARTY_NOTICES.md`.

Build it yourself:

```bash
cd launcher && cargo build --release --bin omni-launcher
```
