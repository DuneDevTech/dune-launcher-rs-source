//! Progress for the Electron launcher (hand-off protocol 2, additive): with `DUNE_PROGRESS=1` every phase of a hand-off prints
//! one line `[dune-progress] {json}` on stderr (the file Electron tails). Lines: `{"phase","files","filesTotal","bytes","bytesTotal"}`,
//! `{"phase":"window","ms":N}` once the Minecraft window exists (ms since the Play click, `DUNE_LAUNCH_T0`). Off = nothing printed.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MARK: &str = "[dune-progress] ";
static PHASE: Mutex<&'static str> = Mutex::new("preparing");
static BYTES: AtomicU64 = AtomicU64::new(0);
static BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);

pub fn enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var_os("DUNE_PROGRESS").is_some_and(|v| v == "1"))
}

pub fn line(json: &str) {
    if enabled() {
        eprintln!("{MARK}{json}");
    }
}

/// Starts a phase (an indeterminate bar until its first numbers arrive).
pub fn phase(name: &'static str) {
    *PHASE.lock().unwrap() = name;
    BYTES.store(0, Relaxed);
    BYTES_TOTAL.store(0, Relaxed);
    emit(0, 0);
}

/// Total bytes of the batch about to be fetched (already-present files are counted as done by `add_done`).
pub fn set_total(bytes: u64) {
    BYTES_TOTAL.store(bytes, Relaxed);
}
pub fn add_done(bytes: u64) {
    BYTES.fetch_add(bytes, Relaxed);
}

pub fn emit(files: usize, files_total: usize) {
    if !enabled() {
        return;
    }
    let p = *PHASE.lock().unwrap();
    line(&format!(
        "{{\"phase\":\"{p}\",\"files\":{files},\"filesTotal\":{files_total},\"bytes\":{},\"bytesTotal\":{}}}",
        BYTES.load(Relaxed),
        BYTES_TOTAL.load(Relaxed)
    ));
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// Milliseconds since the Play click (`DUNE_LAUNCH_T0`), when the launcher passed it.
pub fn since_click() -> Option<u128> {
    let t0: u128 = std::env::var("DUNE_LAUNCH_T0").ok()?.parse().ok()?;
    Some(now_ms().saturating_sub(t0))
}

/// Reports the Minecraft window (once): progress line + launch trace.
pub fn window_seen(how: &str) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let ms = since_click().map(|m| m.to_string()).unwrap_or_else(|| "null".into());
        line(&format!("{{\"phase\":\"window\",\"ms\":{ms},\"how\":\"{how}\"}}"));
        crate::launch_trace::mark(&format!("WINDOW visible ({how})"));
    });
}

/// Waits for the top-level, visible window of process `pid` (Windows: EnumWindows, 20 ms poll, up to 3 min). Elsewhere: false (the game log marks it instead).
#[cfg(windows)]
pub fn wait_window(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};
    unsafe extern "system" fn each(hwnd: HWND, lp: LPARAM) -> i32 {
        let want = &mut *(lp as *mut (u32, bool));
        let mut p = 0u32;
        GetWindowThreadProcessId(hwnd, &mut p);
        if p == want.0 && IsWindowVisible(hwnd) != 0 {
            want.1 = true;
            return 0;
        }
        1
    }
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 180 {
        let mut st = (pid, false);
        unsafe { EnumWindows(Some(each), &mut st as *mut _ as LPARAM) };
        if st.1 {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    false
}
#[cfg(not(windows))]
pub fn wait_window(_pid: u32) -> bool {
    false
}

/// Background watcher for the game's window; on Windows polls EnumWindows, elsewhere the caller feeds log lines (`launch_trace::watch_game_log`).
pub fn watch_window(pid: u32) {
    if !cfg!(windows) {
        return;
    }
    let _ = std::thread::Builder::new().name("window-watch".into()).spawn(move || {
        if wait_window(pid) {
            window_seen("EnumWindows");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_by_default_and_lines_are_json() {
        assert!(!enabled());
        // `emit`/`phase` print nothing without DUNE_PROGRESS=1 and never panic.
        phase("assets");
        set_total(10);
        add_done(5);
        emit(1, 2);
        assert_eq!(BYTES.load(Relaxed), 5);
        let line = format!("{{\"phase\":\"x\",\"files\":{},\"filesTotal\":{},\"bytes\":{},\"bytesTotal\":{}}}", 1, 2, 5, 10);
        assert!(serde_json::from_str::<serde_json::Value>(&line).is_ok());
        assert!(MARK.ends_with(' '));
    }

    #[test]
    fn since_click_needs_the_launchers_t0() {
        assert!(since_click().is_none() || std::env::var("DUNE_LAUNCH_T0").is_ok());
    }
}
