//! Dune Client additions (NOTICE-DUNE.md): the Dune Client mod for each supported Minecraft
//! version, its sha512 pin, and the developer-only offline session used for QA.
//!
//! This module is new code written for Dune Client; it is part of the LGPL-covered launcher.
//! The Dune Client mod itself is a separate program, downloaded at run time from our own host.

use sha2::{Digest, Sha512};
use std::path::{Path, PathBuf};

/// Dune Client never contacts the Horizon project's services (its website, mod registry,
/// telemetry, cosmetics, hosting partner, GitHub organisation): every request the launcher makes
/// passes through here first. `DUNE_ALLOW_UPSTREAM=1` lifts it (tests only).
pub fn upstream_blocked(url: &str) -> bool {
    if std::env::var_os("DUNE_ALLOW_UPSTREAM").is_some() {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    let host = lower
        .split("://")
        .nth(1)
        .unwrap_or(&lower)
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .rsplit('@')
        .next()
        .unwrap_or("");
    host.ends_with("playhorizon.dev")
        || host.ends_with("sculkhosting.com")
        || host.ends_with("discord.gg")
        || (host.ends_with("github.com") || host.ends_with("githubusercontent.com"))
            && lower.contains("horizon-rust")
        || host.ends_with("github.com") && lower.contains("/ludovic111")
}

/// Where the mod jars and the version manifest live (`DUNE_DOWNLOAD_HOST` overrides it for QA).
pub fn download_host() -> String {
    std::env::var("DUNE_DOWNLOAD_HOST")
        .ok()
        .filter(|h| !h.trim().is_empty())
        .unwrap_or_else(|| "duneclient.com".into())
}

/// A Dune Client mod build for one Minecraft version.
#[derive(Clone, Copy, Debug)]
pub struct Build {
    pub minecraft: &'static str,
    pub file: &'static str,
    pub sha512: &'static str,
    /// Fabric Loader the build was verified with.
    pub loader: &'static str,
}

/// The builds shipped with the launcher (same pins as the Electron launcher's `versions.ts`).
pub const BUILDS: [Build; 4] = [
    Build {
        minecraft: "1.21.11",
        file: "DuneClientMod-1.2.5-mc1.21.11.jar",
        sha512: "b61650402086db76513341e9170e62fd406a5ea6395ae3cc22db1cb5377f7521b4a559393208f56198718fc25c0d6fa84f3d757820220357853da3cab275e6f9",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.4",
        file: "DuneClientMod-1.2.5-mc1.21.4.jar",
        sha512: "16ad12cf0457d2a518e8ccbdbedf42e56fc1621bddf3b8a36e63ffc10d9d1f3df0b71b1a496c705234196621a4a1ccc58e9f8acce37cf65e13d768993f49ae15",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.1",
        file: "DuneClientMod-1.2.5-mc1.21.1.jar",
        sha512: "3e309992b0d56425e992de00fe46d58ca78dddc475107cda1af74d71c174077c23be8c2a2691368179e9134d8921fc581f1d59e52fe767be84855e3d71560a2a",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.20.1",
        file: "DuneClientMod-1.2.5-mc1.20.1.jar",
        sha512: "d217024b6b744f36a861d3046e70c39c066fbfec8f755afc629078c55d7fb1d5e0fdc7e25db7fa4b517090f168a4e43750850ccf573ecf058d63d7d98c84e65e",
        loader: "0.20.1",
    },
];

pub fn build_for(minecraft: &str) -> Option<&'static Build> {
    BUILDS.iter().find(|b| b.minecraft == minecraft)
}

/// File name of the managed mod inside an instance's `mods/` folder.
pub const MOD_FILE: &str = "DuneClientMod.jar";

fn sha512_hex(bytes: &[u8]) -> String {
    Sha512::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Makes sure `<game_dir>/mods/DuneClientMod.jar` is the pinned build for `minecraft`:
/// downloads it over https from our host when missing or when its sha512 differs.
/// Other jars in `mods/` are left alone.
pub fn ensure_mod(
    game_dir: &Path,
    minecraft: &str,
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    log: &dyn Fn(String),
) -> Result<PathBuf, String> {
    let build = build_for(minecraft)
        .ok_or_else(|| format!("Dune Client has no mod build for Minecraft {minecraft}"))?;
    let mods = game_dir.join("mods");
    let target = mods.join(MOD_FILE);
    if let Ok(have) = std::fs::read(&target) {
        if sha512_hex(&have) == build.sha512 {
            return Ok(target);
        }
    }
    let url = format!(
        "https://{}/downloads/client/{}",
        download_host(),
        build.file
    );
    log(format!("Downloading Dune Client {minecraft}"));
    let body = fetch(&url)?;
    if sha512_hex(&body) != build.sha512 {
        return Err(format!("{url}: the file does not match its sha512 pin"));
    }
    std::fs::create_dir_all(&mods).map_err(|e| format!("{}: {e}", mods.display()))?;
    let part = mods.join(format!("{MOD_FILE}.part"));
    std::fs::write(&part, &body).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, &target).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target)
}

/// Third-party mods Dune Client needs, never redistributed by us: each one is downloaded from
/// Modrinth (sha512 verified), like the Electron launcher's `THIRD_PARTY_MODS`.
pub const REQUIRED_MODS: [&str; 3] = ["fabric-api", "malilib", "litematica"];
/// Added in the `Dune + Fabric` mode (a mods folder of the player's own, with Mod Menu).
pub const MOD_MENU: &str = "modmenu";
pub const FABRIC_API: &str = "fabric-api";

/// The Minecraft versions Dune Client offers (the ones with a Dune Client mod build).
pub fn supported_versions() -> impl Iterator<Item = &'static str> {
    BUILDS.iter().map(|b| b.minecraft)
}

/// How an instance plays: Dune Client's three modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Dune Client with Fabric kept internal (a managed mods folder).
    Dune,
    /// Dune Client on a Fabric instance with the player's own mods and Mod Menu.
    DuneFabric,
    /// Mojang's game as it is.
    Vanilla,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Dune, Mode::DuneFabric, Mode::Vanilla];
    /// The value of `-Ddune.mode` the mod reads.
    pub fn property(self) -> Option<&'static str> {
        match self {
            Mode::Dune => Some("dune"),
            Mode::DuneFabric => Some("fabric"),
            Mode::Vanilla => None,
        }
    }
}

const MODRINTH: &str = "https://api.modrinth.com/v2";

fn pct(raw: &str) -> String {
    raw.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// The newest release (else beta, else alpha) of `slug` for Fabric on `minecraft`:
/// file name, url, sha512.
fn resolve(
    slug: &str,
    minecraft: &str,
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<(String, String, String), String> {
    let url = format!(
        "{MODRINTH}/project/{slug}/version?game_versions={}&loaders={}",
        pct(&format!("[\"{minecraft}\"]")),
        pct("[\"fabric\"]")
    );
    let body = fetch(&url)?;
    let versions: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| format!("Modrinth {slug}: {e}"))?;
    let rank = |v: &serde_json::Value| match v["version_type"].as_str() {
        Some("release") => 0,
        Some("beta") => 1,
        _ => 2,
    };
    let best = versions
        .as_array()
        .into_iter()
        .flatten()
        .min_by(|a, b| {
            rank(a).cmp(&rank(b)).then_with(|| {
                b["date_published"]
                    .as_str()
                    .cmp(&a["date_published"].as_str())
            })
        })
        .ok_or_else(|| format!("Modrinth: no Fabric build of {slug} for {minecraft}"))?;
    let files = best["files"].as_array().cloned().unwrap_or_default();
    let file = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())
        .ok_or_else(|| format!("Modrinth: {slug} lists no file"))?;
    let name = file["filename"].as_str().unwrap_or_default();
    let url = file["url"].as_str().unwrap_or_default();
    let sha = file["hashes"]["sha512"].as_str().unwrap_or_default();
    if name.is_empty()
        || name.contains(['/', '\\'])
        || !url.starts_with("https://cdn.modrinth.com/")
        || sha.len() != 128
    {
        return Err(format!("Modrinth: {slug} answered an unusable file entry"));
    }
    Ok((name.to_owned(), url.to_owned(), sha.to_ascii_lowercase()))
}

/// Installs [`REQUIRED_MODS`] into `<game_dir>/mods`, tracking what it put there in
/// `.dune-managed.json` so an update replaces the old file and the player's own jars stay alone.
/// A failed lookup keeps the file already there (offline start); missing and failing is an error.
pub fn ensure_required_mods(
    game_dir: &Path,
    minecraft: &str,
    with_mod_menu: bool,
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    log: &dyn Fn(String),
) -> Result<(), String> {
    let wanted: Vec<&str> = REQUIRED_MODS.iter().copied().chain(with_mod_menu.then_some(MOD_MENU)).collect();
    ensure_mods(game_dir, minecraft, &wanted, fetch, log)
}

/// Plain Fabric mode: only the newest stable Fabric API (the player's own jars stay alone).
pub fn ensure_fabric_api(
    game_dir: &Path,
    minecraft: &str,
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    log: &dyn Fn(String),
) -> Result<(), String> {
    ensure_mods(game_dir, minecraft, &[FABRIC_API], fetch, log)
}

fn ensure_mods(
    game_dir: &Path,
    minecraft: &str,
    wanted: &[&str],
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    log: &dyn Fn(String),
) -> Result<(), String> {
    let mods = game_dir.join("mods");
    std::fs::create_dir_all(&mods).map_err(|e| format!("{}: {e}", mods.display()))?;
    let lock = mods.join(".dune-managed.json");
    let mut managed: std::collections::BTreeMap<String, String> = std::fs::read(&lock)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    for &slug in wanted {
        match resolve(slug, minecraft, fetch) {
            Ok((name, url, sha)) => {
                let target = mods.join(&name);
                let current = std::fs::read(&target)
                    .map(|b| sha512_hex(&b) == sha)
                    .unwrap_or(false);
                if !current {
                    log(format!("Downloading {slug} ({name})"));
                    let body = fetch(&url)?;
                    if sha512_hex(&body) != sha {
                        return Err(format!("{url}: the file does not match Modrinth's sha512"));
                    }
                    let part = mods.join(format!("{name}.part"));
                    std::fs::write(&part, &body).map_err(|e| format!("{}: {e}", part.display()))?;
                    std::fs::rename(&part, &target)
                        .map_err(|e| format!("{}: {e}", target.display()))?;
                }
                if let Some(old) = managed.insert(slug.to_owned(), name.clone()) {
                    if old != name {
                        let _ = std::fs::remove_file(mods.join(old));
                    }
                }
            }
            Err(e) => match managed.get(slug) {
                Some(have) if mods.join(have).is_file() => log(format!("{slug}: using {have} ({e})")),
                _ => return Err(e),
            },
        }
    }
    std::fs::write(&lock, serde_json::to_vec_pretty(&managed).unwrap_or_default())
        .map_err(|e| format!("{}: {e}", lock.display()))
}

/// Developer/QA only: `DUNE_DEV_OFFLINE=1` plays with an offline name and skips the Microsoft
/// owner gate. It exists only in debug builds or builds made with `--features dev-offline`;
/// published release builds ignore the variable.
pub fn dev_offline() -> bool {
    cfg!(any(debug_assertions, feature = "dev-offline"))
        && std::env::var_os("DUNE_DEV_OFFLINE").as_deref() == Some(std::ffi::OsStr::new("1"))
}

/// The offline QA player.
pub fn offline_player(name: &str) -> crate::java_launch::Player {
    // A stable, valid-format UUID derived from the name (SHA-512 based, not vanilla's MD5
    // OfflinePlayer UUID): only used for single-player QA.
    let digest = Sha512::digest(format!("OfflinePlayer:{name}").as_bytes());
    let mut b = [0u8; 16];
    b.copy_from_slice(&digest[..16]);
    b[6] = (b[6] & 0x0f) | 0x30;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    crate::java_launch::Player {
        name: name.to_owned(),
        uuid: format!(
            "{}-{}-{}-{}-{}",
            &h[0..8],
            &h[8..12],
            &h[12..16],
            &h[16..20],
            &h[20..32]
        ),
        token: "0".into(),
        xuid: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_build_has_a_wellformed_pin() {
        for b in BUILDS {
            assert_eq!(b.sha512.len(), 128, "{}", b.minecraft);
            assert!(b.sha512.bytes().all(|c| c.is_ascii_hexdigit()));
            assert!(b.file.contains(b.minecraft));
        }
    }

    #[test]
    fn ensure_mod_downloads_checks_and_reuses() {
        let dir = tempfile::tempdir().unwrap();
        let bad = |_: &str| Ok(b"not the jar".to_vec());
        assert!(ensure_mod(dir.path(), "1.21.11", &bad, &|_| {}).is_err());
        assert!(!dir.path().join("mods").join(MOD_FILE).exists());
        assert!(ensure_mod(dir.path(), "1.7.10", &bad, &|_| {}).is_err());
    }

    #[test]
    fn offline_player_is_stable() {
        let a = offline_player("DuneDev");
        assert_eq!(a.uuid, offline_player("DuneDev").uuid);
        assert_eq!(a.uuid.len(), 36);
    }
}

#[cfg(test)]
mod upstream_tests {
    use super::upstream_blocked;

    #[test]
    fn the_horizon_projects_services_are_never_contacted() {
        for url in [
            "https://playhorizon.dev/mods",
            "https://telemetry.playhorizon.dev/v1",
            "https://mcp.playhorizon.dev/github/public/x",
            "https://my.sculkhosting.com/ref/horizon",
            "https://api.github.com/repos/horizon-rust/horizon/releases",
            "https://raw.githubusercontent.com/horizon-rust/horizon/main/launcher/links.json",
        ] {
            assert!(upstream_blocked(url), "{url}");
        }
        for url in [
            "https://duneclient.com/downloads/launcher-rs/latest.json",
            "https://api.modrinth.com/v2/project/fabric-api/version",
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            "https://meta.fabricmc.net/v2/versions/loader/1.21.11",
            "https://api.github.com/repos/someone/mod/releases",
        ] {
            assert!(!upstream_blocked(url), "{url}");
        }
    }
}

/// The licence notices shown in About: our changes, the third-party list, and the licence text.
pub fn licenses_text() -> String {
    format!(
        "{}\n\n---\n\n{}\n\n---\n\n{}\n",
        include_str!("../../NOTICE-DUNE.md"),
        include_str!("../../THIRD_PARTY_NOTICES.md"),
        include_str!("../../LICENSE"),
    )
}

/// Writes [`licenses_text`] to `LICENCIAS.md` in `dir` and returns its path.
pub fn write_licenses(dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("LICENCIAS.md");
    std::fs::write(&path, licenses_text()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}
