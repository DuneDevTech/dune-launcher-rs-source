//! Launch trace for the hot path (Jugar): `DUNE_LAUNCH_TRACE=<file>` + `DUNE_LAUNCH_T0=<epoch ms of the click>` (set by the
//! Electron launcher) make every phase of a hand-off append one line `[+ms since the click] rust: <phase>` to the same file the
//! Electron side writes (`%APPDATA%\.duneclient\logs\launch-trace.txt`). Off (one cached env lookup) when the variables are absent.
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

struct Trace {
    file: PathBuf,
    t0: u128,
}

fn trace() -> Option<&'static Trace> {
    static T: OnceLock<Option<Trace>> = OnceLock::new();
    T.get_or_init(|| {
        let file = PathBuf::from(std::env::var_os("DUNE_LAUNCH_TRACE")?);
        let t0 = std::env::var("DUNE_LAUNCH_T0").ok()?.parse().ok()?;
        Some(Trace { file, t0 })
    })
    .as_ref()
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// Appends `[+ms] who: name` (no-op unless tracing).
pub fn mark_as(who: &str, name: &str) {
    let Some(t) = trace() else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&t.file) {
        let _ = writeln!(f, "[+{}] {who}: {name}", now_ms().saturating_sub(t.t0));
    }
}

pub fn mark(name: &str) {
    mark_as("rust", name);
}

pub fn enabled() -> bool {
    trace().is_some()
}

/// Phases of Minecraft's own log, timed on first sight (poll every 40 ms for up to two minutes, then gone).
const GAME: &[(&str, &str)] = &[
    ("mc-setting-user", "Setting user:"),
    ("mc-loading", "Loading Minecraft"),
    ("mc-backend-lwjgl", "LWJGL version"),
    ("mc-resource-reload", "Reloading ResourceManager"),
    ("mc-sound-engine", "Sound engine started"),
    ("mc-atlas-created", "atlas"),
];

/// Watches the game's stdout file (`DUNE_LAUNCH_GAMELOG`, written by the Electron launcher) for the phases above.
pub fn watch_game_log() {
    let (Some(_), Some(path)) = (trace(), std::env::var_os("DUNE_LAUNCH_GAMELOG")) else { return };
    let _ = std::thread::Builder::new().name("launch-trace".into()).spawn(move || {
        let mut seen = [false; GAME.len()];
        let mut first = false;
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 120 && !seen.iter().all(|s| *s) {
            if let Ok(text) = std::fs::read(&path) {
                let text = String::from_utf8_lossy(&text);
                if !first && !text.is_empty() {
                    first = true;
                    mark_as("game(rust 40ms poll)", "mc-first-line");
                }
                for (i, (name, needle)) in GAME.iter().enumerate() {
                    if !seen[i] && text.contains(needle) {
                        seen[i] = true;
                        mark_as("game(rust 40ms poll)", name);
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
    });
}
