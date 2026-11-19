//! Starting a game through the engine, away from the UI thread. No GPUI in here, so it is testable.
use horizon_launcher::{account, backend, instances::Instance, launch};
use std::sync::mpsc::Sender;

/// What the worker tells the window.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// A progress line (downloads, Java, loaders).
    Line(String),
    /// The game process is running.
    Started,
    /// The game ended; `Err` carries what to show the player.
    Finished(Result<(), String>),
}

/// Whether the new window can start this instance. Instances that play Horizon's own client are not
/// wired into this interface yet; saying so is better than a button that does nothing.
pub fn playable(instance: &Instance) -> Result<(), &'static str> {
    if instance.java.is_some() {
        Ok(())
    } else {
        Err("This instance plays the OMNI Client, which this window cannot start yet.")
    }
}

/// Starts `instance` on a worker thread and reports through `tx`.
pub fn start(instance: Instance, tx: Sender<Event>) {
    std::thread::spawn(move || {
        let result = run(&instance, &tx);
        let _ = tx.send(Event::Finished(result));
    });
}

fn run(instance: &Instance, tx: &Sender<Event>) -> Result<(), String> {
    playable(instance).map_err(str::to_string)?;
    let log = |line: String| {
        let _ = tx.send(Event::Line(line));
    };
    let mut cmd = backend::java_game(instance, None, &log)?;
    if let Some(e) = account::take_unsaved() {
        log(e);
    }
    let vars = instance.vars(cmd.get_program());
    cmd.envs(vars.iter().map(|(k, v)| (k, v)));
    let cmd = launch::wrap(cmd, &instance.launch);
    let (mut cmd, _runtime) = backend::prepare_command(cmd)?;
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not start {:?}: {e}", cmd.get_program()))?;
    let _ = tx.send(Event::Started);
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("The game exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_launcher::instances;

    #[test]
    fn the_default_client_instance_is_refused_with_a_reason() {
        let default = instances::list()
            .into_iter()
            .find(|i| i.java.is_none())
            .expect("the OMNI Client instance always exists");
        assert!(playable(&default).is_err());
    }

    #[test]
    fn a_worker_for_an_unplayable_instance_finishes_with_an_error() {
        let default = instances::list().into_iter().find(|i| i.java.is_none()).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        start(default, tx);
        match rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
            Event::Finished(Err(e)) => assert!(e.contains("OMNI Client")),
            other => panic!("unexpected {other:?}"),
        }
    }
}
