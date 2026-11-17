//! Hand-off from the Electron launcher (Dune Client): `dune-launcher handoff` starts one instance the
//! Electron launcher already prepared the account for. The spec arrives as JSON on **stdin** (never on
//! the command line, where other processes can read it), holding the Minecraft account session; it is
//! read once and dropped. The Electron launcher has already checked that the account owns the game, so
//! this path skips the keychain owner gate but still verifies every download (sha1/sha512).
//!
//! What it shares with the Electron launcher: the instance folder (mods, saves, options), the Minecraft
//! folder (`versions/`, `libraries/`, `assets/`, `runtime/`: same layout, so a warm cache is warm for
//! both) and the managed-mods lock file (`mods/.dune-managed.json`).
use crate::instances::{Instance, JavaGame, Kind};
use crate::java_launch::Player;
use crate::loaders::{Kind as LoaderKind, Loader};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bumped when the spec changes; the Electron launcher only hands off to a binary that reports it.
pub const PROTOCOL: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Spec {
    pub protocol: u32,
    pub minecraft: String,
    /// `dune`, `fabric` (Dune + Fabric) or `vanilla`.
    pub mode: String,
    pub game_dir: PathBuf,
    pub minecraft_dir: PathBuf,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub java_home: Option<PathBuf>,
    #[serde(default)]
    pub window: Option<(u32, u32)>,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub server: Option<String>,
    /// Program and arguments to start once the game has ended (the Electron launcher uses it to come back
    /// after it quit to free its memory while you play). `--dune-after-game=<exit code>` is appended.
    #[serde(default)]
    pub after_exit: Option<Vec<String>>,
    pub player: SpecPlayer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpecPlayer {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    #[serde(default)]
    pub xuid: String,
}

#[derive(Serialize)]
pub struct Info {
    pub protocol: u32,
    pub version: String,
    /// Minecraft versions with a Dune Client mod build this binary can install itself.
    pub dune_versions: Vec<String>,
}

pub fn info() -> Info {
    Info {
        protocol: PROTOCOL,
        version: env!("CARGO_PKG_VERSION").to_string(),
        dune_versions: crate::dune::supported_versions().map(String::from).collect(),
    }
}

pub fn parse(json: &str) -> Result<Spec, String> {
    let spec: Spec = serde_json::from_str(json).map_err(|e| format!("hand-off spec: {e}"))?;
    if spec.protocol != PROTOCOL {
        return Err(format!("hand-off protocol {} is not {PROTOCOL}", spec.protocol));
    }
    if !matches!(spec.mode.as_str(), "dune" | "fabric" | "vanilla") {
        return Err(format!("hand-off mode {:?} is not dune, fabric or vanilla", spec.mode));
    }
    if !spec.game_dir.is_absolute() || !spec.minecraft_dir.is_absolute() {
        return Err("hand-off folders must be absolute paths".into());
    }
    if let Some(after) = &spec.after_exit {
        if after.first().is_none_or(|program| !std::path::Path::new(program).is_absolute()) {
            return Err("hand-off afterExit must start with an absolute program path".into());
        }
    }
    if spec.player.access_token.is_empty() || spec.player.name.is_empty() {
        return Err("hand-off needs a player name and token".into());
    }
    Ok(spec)
}

/// The instance the spec describes.
pub fn instance(spec: &Spec) -> Result<Instance, String> {
    let dune_build = crate::dune::build_for(&spec.minecraft);
    let (dune, dune_fabric, loader) = match spec.mode.as_str() {
        "dune" | "fabric" if dune_build.is_none() => {
            return Err(format!("Dune Client has no mod build for Minecraft {} in this launcher", spec.minecraft))
        }
        "dune" => (true, false, true),
        "fabric" => (false, true, true),
        _ => (false, false, false),
    };
    let loader = loader.then(|| Loader {
        kind: LoaderKind::Fabric,
        version: spec
            .loader_version
            .clone()
            .or_else(|| dune_build.map(|b| b.loader.to_string()))
            .unwrap_or_default(),
    });
    if loader.as_ref().is_some_and(|l| l.version.is_empty()) {
        return Err("hand-off needs a Fabric loader version".into());
    }
    Ok(Instance {
        id: "handoff".into(),
        name: "Dune Client".into(),
        kind: Kind::Own,
        work: spec.game_dir.clone(),
        home: spec.game_dir.clone(),
        icon: None,
        group: None,
        version: None,
        channel: Default::default(),
        native_profile: crate::native_mods::Profile::Optimized,
        played: Default::default(),
        last_played: None,
        launch: Default::default(),
        java: Some(JavaGame {
            minecraft: spec.minecraft.clone(),
            home: spec.java_home.clone(),
            memory_mb: spec.memory_mb,
            jvm_args: spec.jvm_args.join(" "),
            loader,
            window: spec.window,
            dune,
            dune_fabric,
        }),
    })
}

/// Starts the `afterExit` program (not waited for), telling it how the game ended.
pub fn run_after_exit(spec: &Spec, code: i32) {
    let Some(after) = &spec.after_exit else { return };
    let mut cmd = std::process::Command::new(&after[0]);
    cmd.args(&after[1..])
        .arg(format!("--dune-after-game={code}"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if let Err(e) = cmd.spawn() {
        eprintln!("[dune-launcher] could not start {}: {e}", after[0]);
    }
}

pub fn player(spec: &Spec) -> Player {
    Player {
        name: spec.player.name.clone(),
        uuid: spec.player.uuid.clone(),
        token: spec.player.access_token.clone(),
        xuid: spec.player.xuid.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(extra: &str) -> String {
        let (game, mc) = if cfg!(windows) { ("C:\\g", "C:\\m") } else { ("/g", "/m") };
        let (game, mc) = (game.replace('\\', "\\\\"), mc.replace('\\', "\\\\"));
        format!(
            r#"{{"protocol":1,"minecraft":"1.21.11","mode":"dune","gameDir":"{game}","minecraftDir":"{mc}",
               "player":{{"name":"Steve","uuid":"00000000-0000-0000-0000-000000000000","accessToken":"SECRETTOKEN123"}}{extra}}}"#
        )
    }

    #[test]
    fn parses_a_dune_spec_and_never_prints_the_token() {
        let spec = parse(&json("")).unwrap();
        assert_eq!(spec.mode, "dune");
        let player = player(&spec);
        assert!(!format!("{player:?}").contains("SECRETTOKEN123"));
        let instance = instance(&spec).unwrap();
        let game = instance.java.unwrap();
        assert!(game.dune && !game.dune_fabric);
        assert_eq!(game.loader.unwrap().kind, LoaderKind::Fabric);
    }

    #[test]
    fn rejects_unknown_fields_modes_protocols_and_relative_paths() {
        assert!(parse(&json(r#","extra":1"#)).is_err());
        assert!(parse(&json("").replace("\"dune\"", "\"hax\"")).is_err());
        assert!(parse(&json("").replace("\"protocol\":1", "\"protocol\":9")).is_err());
        assert!(parse(&json("").replace("\"gameDir\":\"", "\"gameDir\":\"rel")).is_err());
        assert!(parse(&json("").replace("SECRETTOKEN123", "")).is_err());
    }

    #[test]
    fn fabric_mode_is_dune_fabric_and_vanilla_has_no_loader() {
        let fabric = instance(&parse(&json("").replace("\"dune\"", "\"fabric\"")).unwrap()).unwrap();
        assert!(fabric.java.as_ref().unwrap().dune_fabric);
        let vanilla = instance(&parse(&json("").replace("\"dune\"", "\"vanilla\"")).unwrap()).unwrap();
        assert!(vanilla.java.unwrap().loader.is_none());
    }

    #[test]
    fn a_version_without_a_dune_build_is_refused() {
        let spec = parse(&json("").replace("1.21.11", "1.8.9")).unwrap();
        assert!(instance(&spec).is_err());
    }

    #[test]
    fn a_loader_profile_without_artifacts_is_not_usable() {
        let dir = std::env::temp_dir().join(format!("handoff-profile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (plain, ours, junk) = (dir.join("plain.json"), dir.join("ours.json"), dir.join("junk.json"));
        std::fs::write(&plain, r#"{"libraries":[{"name":"net.fabricmc:fabric-loader:0.20.1","url":"https://maven.fabricmc.net/"}]}"#).unwrap();
        std::fs::write(&ours, r#"{"libraries":[{"name":"a:b:1","downloads":{"artifact":{"path":"a/b/1/b-1.jar"}}}]}"#).unwrap();
        std::fs::write(&junk, "not json").unwrap();
        assert!(!crate::backend::loader_profile_usable(&plain));
        assert!(crate::backend::loader_profile_usable(&ours));
        assert!(!crate::backend::loader_profile_usable(&junk));
        assert!(!crate::backend::loader_profile_usable(&dir.join("missing.json")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn after_exit_needs_an_absolute_program_and_is_appended_the_exit_code() {
        let abs = if cfg!(windows) { "C:\\\\x.exe" } else { "/bin/true" };
        let ok = parse(&json(&format!(r#","afterExit":["{abs}","--a"]"#))).unwrap();
        assert_eq!(ok.after_exit.as_ref().unwrap().len(), 2);
        assert!(parse(&json(r#","afterExit":["relative"]"#)).is_err());
        assert!(parse(&json(r#","afterExit":[]"#)).is_err());
        run_after_exit(&ok, 0); // spawns /bin/true (or fails quietly): must not panic
    }

    #[test]
    fn info_lists_the_dune_versions() {
        assert!(info().dune_versions.iter().any(|v| v == "1.21.11"));
    }
}
