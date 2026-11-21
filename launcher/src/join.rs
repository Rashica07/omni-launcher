//! Joining a server that OMNIServers hands over: the Mac app writes `join-request.json` into the
//! launcher's data folder and opens the launcher; the launcher shows the Join popup (which
//! instance, whether to remember it for this server, an invite for friends) and, on Launch,
//! makes or reuses the instance, adds the server to its Multiplayer list and plays it.
//!
//! This module is the part with no window: reading the request, the saved instance per server,
//! the invite text, and putting the server into an instance. The popup and the CLI both use it.
use crate::instances::{self, Instance};
use crate::server_list;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The file OMNIServers writes (and the launcher consumes) in the data folder.
pub const REQUEST_FILE: &str = "join-request.json";
/// What the launcher remembers: a server's address -> the instance it plays on.
pub const SAVED_FILE: &str = "joined-servers.json";
/// A request older than this is stale (the launcher was not open to see it): ignored.
pub const FRESH_SECS: u64 = 600;

/// What OMNIServers asks for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// The server's name in OMNIServers.
    pub name: String,
    /// The address on this computer (`localhost:25565`).
    pub address: String,
    /// The address friends use from outside (Playit), when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_address: Option<String>,
    /// The Minecraft version the server runs (`1.21.4`).
    pub minecraft: String,
    /// When it was written, in seconds since 1970.
    pub created: u64,
}

impl Request {
    /// A request is usable with a name, an address and a version.
    fn valid(&self) -> bool {
        [&self.name, &self.address, &self.minecraft]
            .iter()
            .all(|s| !s.trim().is_empty())
    }

    /// The address friends should use: the public one, else the one on this computer.
    pub fn invite_address(&self) -> &str {
        self.public_address
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .unwrap_or(&self.address)
    }

    /// Whether friends outside this network can use [`Request::invite_address`].
    pub fn has_public_address(&self) -> bool {
        self.public_address
            .as_deref()
            .is_some_and(|a| !a.trim().is_empty())
    }

    /// The text to send a friend.
    pub fn invite_text(&self) -> String {
        format!(
            "Join my Minecraft server \"{}\": {} (Java Edition {})",
            self.name.trim(),
            self.invite_address(),
            self.minecraft.trim()
        )
    }
}

/// The request waiting in `dir`, if a fresh and valid one is there. The file is removed either
/// way, so a broken or stale one never comes back.
pub fn take(dir: &Path, now: u64) -> Option<Request> {
    let path = dir.join(REQUEST_FILE);
    let text = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(&path);
    let request: Request = serde_json::from_str(&text).ok()?;
    let fresh = now.saturating_sub(request.created) <= FRESH_SECS;
    (request.valid() && fresh).then_some(request)
}

/// Whether a request file is waiting (a cheap check for the launcher's poll).
pub fn waiting(dir: &Path) -> bool {
    dir.join(REQUEST_FILE).is_file()
}

/// Seconds since 1970.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn saved_path(dir: &Path) -> PathBuf {
    dir.join(SAVED_FILE)
}

/// The instance id remembered for the server at `address`.
pub fn saved(dir: &Path, address: &str) -> Option<String> {
    let text = std::fs::read_to_string(saved_path(dir)).ok()?;
    let map: std::collections::BTreeMap<String, String> = serde_json::from_str(&text).ok()?;
    map.get(address.trim()).cloned()
}

/// Remembers that the server at `address` plays on instance `id`.
pub fn save(dir: &Path, address: &str, id: &str) -> Result<(), String> {
    let mut map: std::collections::BTreeMap<String, String> =
        std::fs::read_to_string(saved_path(dir))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
    map.insert(address.trim().to_string(), id.to_string());
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    std::fs::write(saved_path(dir), json).map_err(|e| e.to_string())
}

/// The Java instances that play `minecraft`, the one saved for the server first.
pub fn candidates(all: &[Instance], minecraft: &str, saved_id: Option<&str>) -> Vec<Instance> {
    let mut found: Vec<Instance> = all
        .iter()
        .filter(|i| i.java.as_ref().is_some_and(|j| j.minecraft == minecraft.trim()))
        .cloned()
        .collect();
    found.sort_by_key(|i| Some(i.id.as_str()) != saved_id);
    found
}

/// `OMNI: <server>`, or with a number after it when an instance already has that name.
pub fn new_instance_name(all: &[Instance], server: &str) -> String {
    let base = format!("OMNI: {}", server.trim());
    let taken = |n: &str| all.iter().any(|i| i.name.eq_ignore_ascii_case(n));
    if !taken(&base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base} {n}"))
        .find(|n| !taken(n))
        .unwrap()
}

/// Puts the server into the Multiplayer list in `home`, unless its address is already there.
/// Returns whether it was added.
pub fn add_server(home: &Path, name: &str, address: &str) -> Result<bool, String> {
    let mut list = server_list::List::load(home);
    let address = address.trim();
    if list.servers().iter().any(|s| s.ip.trim() == address) {
        return Ok(false);
    }
    list.add(name, address)?;
    list.save(home)?;
    Ok(true)
}

/// Which instance the server plays on.
#[derive(Clone, Debug)]
pub enum Pick {
    /// One that exists.
    Existing(Box<Instance>),
    /// A new one on the server's version.
    New,
}

/// What Launch does before the game starts: makes or takes the instance, puts the server in its
/// list, selects it, and (when `remember`) saves it for this server. `data` is the launcher's
/// data folder. Returns the instance.
pub fn prepare(
    request: &Request,
    pick: Pick,
    remember: bool,
    data: &Path,
) -> Result<Instance, String> {
    let all = instances::list();
    let version = crate::minecraft_versions::load()?
        .find(request.minecraft.trim())?
        .id
        .clone();
    let instance = match pick {
        Pick::Existing(i) => *i,
        Pick::New => {
            let name = new_instance_name(&all, &request.name);
            instances::create_java(&name, &version)?
        }
    };
    add_server(&instance.home, &request.name, &request.address)?;
    let instance = instances::select(&instance.id)?;
    if remember {
        save(data, &request.address, &instance.id)?;
    }
    Ok(instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp() -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "omni-join-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn request(created: u64) -> Request {
        Request {
            name: "Survival".into(),
            address: "localhost:25565".into(),
            public_address: Some("play.example.gg".into()),
            minecraft: "1.21.4".into(),
            created,
        }
    }

    fn write(dir: &Path, r: &Request) {
        std::fs::write(dir.join(REQUEST_FILE), serde_json::to_string(r).unwrap()).unwrap();
    }

    #[test]
    fn a_fresh_request_is_taken_once() {
        let dir = temp();
        write(&dir, &request(1000));
        assert!(waiting(&dir));
        assert_eq!(take(&dir, 1010), Some(request(1000)));
        assert!(!waiting(&dir), "the file is consumed");
        assert_eq!(take(&dir, 1010), None);
    }

    #[test]
    fn a_stale_request_is_ignored_and_removed() {
        let dir = temp();
        write(&dir, &request(1000));
        assert_eq!(take(&dir, 1000 + FRESH_SECS + 1), None);
        assert!(!waiting(&dir));
    }

    #[test]
    fn garbage_and_incomplete_requests_are_ignored() {
        let dir = temp();
        std::fs::write(dir.join(REQUEST_FILE), "not json").unwrap();
        assert_eq!(take(&dir, 1), None);
        let mut r = request(1);
        r.minecraft = "  ".into();
        write(&dir, &r);
        assert_eq!(take(&dir, 1), None);
        assert!(!waiting(&dir));
    }

    #[test]
    fn the_invite_prefers_the_public_address() {
        let mut r = request(1);
        assert_eq!(r.invite_address(), "play.example.gg");
        assert!(r.has_public_address());
        assert!(r.invite_text().contains("play.example.gg"));
        assert!(r.invite_text().contains("1.21.4"));
        r.public_address = Some("  ".into());
        assert_eq!(r.invite_address(), "localhost:25565");
        assert!(!r.has_public_address());
    }

    #[test]
    fn the_saved_instance_round_trips_per_server() {
        let dir = temp();
        assert_eq!(saved(&dir, "a:1"), None);
        save(&dir, "a:1", "one").unwrap();
        save(&dir, " b:2 ", "two").unwrap();
        save(&dir, "a:1", "three").unwrap();
        assert_eq!(saved(&dir, "a:1").as_deref(), Some("three"));
        assert_eq!(saved(&dir, "b:2").as_deref(), Some("two"));
    }

    #[test]
    fn the_server_is_added_to_the_list_once() {
        let home = temp();
        assert!(add_server(&home, "Survival", "localhost:25565").unwrap());
        assert!(!add_server(&home, "Survival", " localhost:25565 ").unwrap());
        let list = server_list::List::load(&home).servers();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Survival");
        assert_eq!(list[0].ip, "localhost:25565");
    }

    #[test]
    fn a_new_instance_name_avoids_the_taken_ones() {
        assert_eq!(new_instance_name(&[], " Survival "), "OMNI: Survival");
    }
}
