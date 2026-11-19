//! OMNI Client's opening film as a native mod (`native-mods/omni-intro`, docs/native-mods.md). The
//! launcher carries the bundle's files, packs them into the `.hnmod` the Mods page knows, and puts
//! it in an instance on request: the player then builds the game with it like with any native mod
//! (the trust warning and the build are the existing ones). Nothing is built or run here.
use horizon_launcher::browser::java;
use horizon_launcher::native_mods::{INSTALLED, MIXINS_REV};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The mod's id (`horizon-mixin.toml`'s `[mod] id`), the name it has in `horizon-mods.txt`.
pub const ID: &str = "omni_intro";
const VERSION: &str = "0.1.0";
/// The game it was made for.
const GAME: &str = "26.3";
/// The Horizon commit the hooks were checked on (upstream main when the mod was made). The mod
/// names the hook version it needs (`HOOKS`), so it builds on any Horizon with that or a later one.
const TESTED_HORIZON: &str = "1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9";
/// `NATIVE_HOOKS_API` the mod needs: `frame` and `gui_render_post` (docs/native-mods.md, API 410, 411).
const HOOKS: u32 = 411;

/// The bundle's files by their place under `mixins/omni_intro/`.
const FILES: [(&str, &[u8]); 6] = [
    ("horizon-mixin.toml", include_bytes!("../../native-mods/omni-intro/horizon-mixin.toml")),
    ("frame.rs", include_bytes!("../../native-mods/omni-intro/frame.rs")),
    ("post.rs", include_bytes!("../../native-mods/omni-intro/post.rs")),
    ("engine/Cargo.toml", include_bytes!("../../native-mods/omni-intro/engine/Cargo.toml")),
    ("engine/src/lib.rs", include_bytes!("../../native-mods/omni-intro/engine/src/lib.rs")),
    ("engine/data/frames.bin", include_bytes!("../../native-mods/omni-intro/engine/data/frames.bin")),
];

/// The `.hnmod`: the manifest and the bundle under `mixins/omni_intro/`.
pub fn hnmod() -> Vec<u8> {
    let manifest = serde_json::json!({
        "schema": 1, "id": ID, "version": VERSION, "game": GAME,
        "horizon": TESTED_HORIZON, "mixins": MIXINS_REV, "hooks": HOOKS,
    });
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file(horizon_mods::index::native::MANIFEST, options).expect("zip entry");
    zip.write_all(manifest.to_string().as_bytes()).expect("zip write");
    for (name, bytes) in FILES {
        zip.start_file(format!("mixins/{ID}/{name}"), options).expect("zip entry");
        zip.write_all(bytes).expect("zip write");
    }
    zip.finish().expect("zip finish").into_inner()
}

fn file(home: &Path) -> PathBuf {
    home.join(INSTALLED).join(format!("{ID}.hnmod"))
}

/// Whether the intro is in the instance whose game folder is `home` (turned on or not).
pub fn installed(home: &Path) -> bool {
    file(home).is_file()
}

/// Puts the intro in `home` and turns it on. The game plays it after the next build.
pub fn install(home: &Path) -> Result<(), String> {
    let path = file(home);
    std::fs::create_dir_all(path.parent().expect("has a parent")).map_err(|e| e.to_string())?;
    std::fs::write(&path, hnmod()).map_err(|e| format!("{}: {e}", path.display()))?;
    java::set_enabled(home, ID, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_launcher::native_mods;

    fn home() -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("omni-intro-mod-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_package_is_a_valid_native_mod() {
        let dir = home();
        let path = dir.join("omni_intro.hnmod");
        std::fs::write(&path, hnmod()).unwrap();
        let bundle = horizon_mods::index::native::Bundle::read(&path).expect("reads as a registry bundle");
        bundle.manifest.check().expect("manifest");
        assert_eq!((bundle.manifest.id.as_str(), bundle.manifest.hooks), (ID, Some(HOOKS)));
        for (name, _) in FILES {
            assert!(bundle.files.contains_key(&format!("mixins/{ID}/{name}")), "{name}");
        }
    }
    #[test]
    fn installing_turns_it_on_and_the_build_unpacks_it() {
        let home = home();
        assert!(!installed(&home));
        assert!(native_mods::installed_mods(&home).is_err(), "nothing on yet");
        install(&home).unwrap();
        assert!(installed(&home) && java::is_enabled(&home, ID));
        let folder = native_mods::installed_mods(&home).unwrap();
        let found = native_mods::bundles(&folder).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, ID);
    }
    #[test]
    fn installing_twice_changes_nothing() {
        let home = home();
        install(&home).unwrap();
        install(&home).unwrap();
        let on = std::fs::read_to_string(home.join("horizon-mods.txt")).unwrap();
        assert_eq!(on.matches("mod.omni_intro:true").count(), 1);
    }
}
