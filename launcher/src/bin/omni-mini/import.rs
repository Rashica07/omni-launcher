//! Bringing instances in from Prism, MultiMC, CurseForge, the official launcher and others:
//! Horizon's own scanner and importer, run off the UI thread.
use horizon_launcher::instances::Instance;
use horizon_launcher::other_launchers::{self, Found};
use horizon_launcher::sources::{Folders, Os};
use std::sync::mpsc::Sender;

/// Every instance the other launchers on this computer keep in their default folders.
pub fn scan() -> Vec<Found> {
    Folders::current().map(|f| other_launchers::scan(Os::current(), &f)).unwrap_or_default()
}

/// Instances inside a folder the player picked (a Prism `instances/` folder, one instance, ...).
#[cfg(test)]
pub fn scan_folder(dir: &std::path::Path) -> Vec<Found> {
    other_launchers::in_folder(dir, None)
}

/// One line for the list: where it came from and what it plays.
pub fn describe(found: &Found) -> String {
    let game = found.minecraft.as_deref().unwrap_or("version unknown");
    match &found.loader {
        Some(l) => format!("{} · Minecraft {game} · {} {}", found.launcher, l.name, l.version),
        None => format!("{} · Minecraft {game}", found.launcher),
    }
}

/// Copies `found` into a new instance on a worker thread. The result goes to `tx`.
pub fn start(found: Found, tx: Sender<Result<Instance, String>>) {
    std::thread::spawn(move || {
        let name = found.name.clone();
        let _ = tx.send(other_launchers::import(&found, &name, None));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn found(minecraft: Option<&str>) -> Found {
        Found {
            launcher: "Prism",
            name: "Pack".into(),
            minecraft: minecraft.map(str::to_string),
            loader: None,
            game: PathBuf::from("/nowhere"),
            shared: false,
        }
    }

    #[test]
    fn describe_names_the_source_and_version() {
        assert_eq!(describe(&found(Some("1.21.1"))), "Prism · Minecraft 1.21.1");
        assert_eq!(describe(&found(None)), "Prism · Minecraft version unknown");
    }

    #[test]
    fn an_empty_folder_holds_no_instances() {
        let dir = std::env::temp_dir().join("omni-empty-scan");
        let _ = std::fs::create_dir_all(&dir);
        assert!(scan_folder(&dir).is_empty());
    }

    #[test]
    fn importing_without_a_known_version_reports_it() {
        let (tx, rx) = std::sync::mpsc::channel();
        start(found(None), tx);
        let r = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(r.is_err());
    }
}

#[cfg(test)]
mod machine {
    /// Prints what the scanner finds on this computer (run with --ignored --nocapture).
    #[test]
    #[ignore]
    fn list_found() {
        for f in super::scan() {
            println!("{} | {}", f.name, super::describe(&f));
        }
    }
}
