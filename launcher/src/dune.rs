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

/// The builds shipped with the launcher (generated from the Electron launcher's `versions.ts` by dune-release/release-1.3.0/stage-1.3.0.py).
pub const BUILDS: &[Build] = &[
    Build {
        minecraft: "26.3",
        file: "DuneClientMod-1.3.0-r3-full-mc26.3.jar",
        sha512: "f540b95276ee1efc13b134931b16793410dfa6d68f0ef6951bb54232c8319eb5d7fbe349374c76d1678c8032a663132b100fb33da716be542b639ff59418b85b",
        loader: "0.20.1",
    },
    Build {
        minecraft: "26.2",
        file: "DuneClientMod-1.3.0-r3-full-mc26.2.jar",
        sha512: "1ac52e3350a0df260685b658fe94957387ff505d7d3bcd0f39c860da3259e6ef5326abb7214b47dc33fcddb6c16eaabd602ac8495b316c054b426c7b55dfebff",
        loader: "0.20.1",
    },
    Build {
        minecraft: "26.1.2",
        file: "DuneClientMod-1.3.0-r3-full-mc26.1.2.jar",
        sha512: "065b297fc5b194826c7f2fcd01ede64e4291cfdc791189475e1751ba7add7a64b33e0ffa1860387e5e2469043f6d19659b9897575434e20cb36b57e01d7e6363",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.11",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.11.jar",
        sha512: "43b84b617848fc046d7ec8e45149d38515d92440f69c7c62cb340be37391bc8b32567e5107893b8cc8f100c71e9391ce63e57efda88d12c4ac1ce5f65f5c9bda",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.10",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.10.jar",
        sha512: "f91f3c99ac7f8c754a71841a6821f432025391058b3653362a939a18794b9667971bb9cae58c13afad128ad2b2e0059f7cd412d949404933d476c82dde5a574c",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.8",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.8.jar",
        sha512: "6856085e699cc85ccc34e6bdd4a237c500016f79e91aaa3376dd9931dc71d7f914034504e059032e9a59f3f937254d8413283dfa82734d43299d9a1b850bbe8b",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.7",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.7.jar",
        sha512: "f6171f5fe3b5a4fa35b1abd03b299d296799e940e9ff46ddcb2eecd0fd7c4844ee41aaaffb44cc606e8c1e38c44a453d074402d842d4508566555f214d4b095c",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.6",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.6.jar",
        sha512: "659d9859f295208cab5bf3206d96a838e96c57ba9954be388d142901f2490295fe26677e7d3224171f7655ec56fc908754deaf0b8b81a47f09f018bd49ba1692",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.5",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.5.jar",
        sha512: "dc6bc59d772c523f89f9c9f7671b7123926d94ce2236bd517416b2461292003c1e27cb82976afb6629d53deccf8f325cfeb8acd08d44f0af7c990b4b22b65a82",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.4",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.4.jar",
        sha512: "89e348e94bb38b0f6f7b9ff3a33544d04c0e0afd09ca53f27cbc95cff3a7c3f93cc2b54eb8446c64c2c01120981aa3bda6b446cdd5efa1ee8a9e87fbffb204e5",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.3",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.3.jar",
        sha512: "62d0c5c93b715804c21cbd044f132fb5c1584985551b589b60e284524908472a053a7ebd26d83ef2bfa22d064b4f5f7541385de76e39aa28690a2dde907e5466",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.2.jar",
        sha512: "e04ba40e274ad25f40412e6b0b6dcef18e0c69ec0c8234e4bafb8a4c05dc470dcacf3264f14e02afdf56166a73cec7f13abaf91f791b1e2fda931a249e38f4e6",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21.1",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.1.jar",
        sha512: "bf7667b4ec327931f218951a12e7f7f1e8e024c379afe428f5fd1040b2dc9a24f4d4faae5ce9b39834878b308010688dff53207cdb23b0752eb90f075cc81cbe",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.21",
        file: "DuneClientMod-1.3.0-r3-full-mc1.21.jar",
        sha512: "26bade0b2d7441e3fd349502930fad8835abc78ce0c5197e8580c70cf86f8e395d29ae837b539cb398a6313aa81014205165987c2ac5db842845ff674b4434a7",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.20.6",
        file: "DuneClientMod-1.3.0-r3-full-mc1.20.6.jar",
        sha512: "69f14c0a9eaa5b4a77fb12a27a53652ab97520dd919d28d3fe937b5413951f4621cbd4feb61e5cbadfe1feb6ae82bf13ebc542a76c48012a377482a4079a965a",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.20.4",
        file: "DuneClientMod-1.3.0-r3-full-mc1.20.4.jar",
        sha512: "8599996296c18a692010b157df74a11f5cfaba3a067528205025b98457a40fffa53d2fa6f1845965c5076f83a9bf857e3e98bd910cc9f9b3ca14f8b8b61e22d0",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.20.1",
        file: "DuneClientMod-1.3.0-r3-full-mc1.20.1.jar",
        sha512: "776c70873e5a19cb6d9190fb9570c31c784403ba0a6387daea4e5978f5d8637fe5419755a53ac44c74271729ddd2dd658415509f6ee43c23e1b498dfdd527bf1",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.19.4",
        file: "DuneClientMod-1.3.0-r3-full-mc1.19.4.jar",
        sha512: "d5a71f47e5e5cd03443ba41bdb66a0084c8e0abb2d2cdc628f92cd1ca048475cfd4f06b1258881763d791222cfb63a14c96631644df25495dafab802ccd41ab3",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.19.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.19.2.jar",
        sha512: "39930c72652a4e69f35fc0ac6631e079dd0fb4847d8e3a9838a45ac7e8aa628a8deefd33409b6f927ee8b41b6cef3a824838e4adcb332b1731c28c7f923600de",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.18.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.18.2.jar",
        sha512: "303bfc2e5f7b8b9913e5e5005ee960854255a9e0becbdf4683c2a3f9cffd07a33bf222b1dceff10f69becca63c72188cbfb5cde632cde9e9baf1c7d3054dd390",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.17.1",
        file: "DuneClientMod-1.3.0-r3-full-mc1.17.1.jar",
        sha512: "3d11f6cec08827b8bf06ccaff09ba40263055f27d7fb92614cf855fd4f9f47a5603f050aebfd33c100d768841e659f6f9a786740d23d8cc7bedf44ca1a1474c5",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.16.5",
        file: "DuneClientMod-1.3.0-r3-full-mc1.16.5.jar",
        sha512: "25dfe9db15cf9ab972897a27d0f5777d15ecbfb7b87c0cabfc8f68a274ae71c85b63c128b051180c5d1704b2c7f082bc92161d3401f9f59dfc7c28c9ce8f19ac",
        loader: "0.20.1",
    },
    Build {
        minecraft: "1.12.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.12.2.jar",
        sha512: "7b1ad3b443ecd7f6211170bd82a01aed7721d2a9e257a776b45590ab8d686dc56f9a3fca8e991c1216d3c8fd3186047b49e8828d6b59ee153604981f085f95d7",
        loader: "0.18.0",
    },
    Build {
        minecraft: "1.11.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.11.2.jar",
        sha512: "b74c1c1a30da0976e414700ded330038929fcda1929db79aeb96f95bfc6a364abc4814aa5dc7f1588ebcae8568a2f67148af6c0874f73fa40b060d1e3d945b14",
        loader: "0.18.0",
    },
    Build {
        minecraft: "1.10.2",
        file: "DuneClientMod-1.3.0-r3-full-mc1.10.2.jar",
        sha512: "cdb2c96c5474bbf5e8bd992fe10e28cc82bce717cf1f070e2ff8d3b828fac14f9a468772bccc21656fe73df3b8b326138d82b2512a01b2f2db3b164ee07624f2",
        loader: "0.18.0",
    },
    Build {
        minecraft: "1.9.4",
        file: "DuneClientMod-1.3.0-r3-full-mc1.9.4.jar",
        sha512: "40044e4fb315d566cd1e93e9c8a916ac4fd2f966d441b040335a15d59393766b1e06e67515bb3719cd7e6f5d50af0494013aa3c8fc8b61f2213c182bf242cc91",
        loader: "0.18.0",
    },
    Build {
        minecraft: "1.8.9",
        file: "DuneClientMod-1.3.0-r3-full-mc1.8.9.jar",
        sha512: "334c40bb68809e322559b3b6b5d1ba46bf46dbb219de231d8ec68be0fc37467a9c7906f95a29c5bf850dfae26ca55550c2ead138275a8ba3b7bbb24790147e45",
        loader: "0.18.0",
    },
];

/// Minecraft 1.8-1.12 (Legacy Fabric): no Fabric API / Mod Menu on Modrinth, the Dune jar is self-contained.
pub fn is_legacy_fabric(minecraft: &str) -> bool {
    let mut parts = minecraft.split('.');
    parts.next() == Some("1") && parts.next().and_then(|m| m.parse::<u32>().ok()).is_some_and(|m| (8..=12).contains(&m))
}

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


const SIGS_FILE: &str = ".dune-sigs.json";

/// sha512 of a file, remembered per size + mtime in `<folder>/.dune-sigs.json`: a repeat launch does not read and hash
/// the same multi-MB jars again (slow under Windows Defender); a changed file (size or mtime) is hashed again.
fn sha512_of_file(path: &Path) -> Option<String> {
    let md = std::fs::metadata(path).ok()?;
    let mtime = md.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos() as u64;
    let name = path.file_name()?.to_str()?.to_owned();
    let sigs_path = path.parent()?.join(SIGS_FILE);
    let mut sigs: serde_json::Map<String, serde_json::Value> = std::fs::read(&sigs_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if let Some(e) = sigs.get(&name) {
        if e[0].as_u64() == Some(md.len()) && e[1].as_u64() == Some(mtime) {
            if let Some(h) = e[2].as_str() {
                return Some(h.to_owned());
            }
        }
    }
    let hash = sha512_hex(&std::fs::read(path).ok()?);
    sigs.insert(name, serde_json::json!([md.len(), mtime, hash]));
    let _ = std::fs::write(&sigs_path, serde_json::to_vec(&sigs).unwrap_or_default());
    Some(hash)
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
    if sha512_of_file(&target).as_deref() == Some(build.sha512) {
        return Ok(target);
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
    if is_legacy_fabric(minecraft) {
        return Ok(());
    }
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
    if is_legacy_fabric(minecraft) {
        return Ok(());
    }
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
    // Fast path: every wanted mod was resolved and hash-verified recently for this Minecraft version and is
    // still on disk with the same sha512 -> no Modrinth round trips (they sat in front of every launch).
    if verified_recently(&mods, minecraft, wanted) {
        return Ok(());
    }
    let mut verified: std::collections::BTreeMap<String, (String, String)> = Default::default();
    let lock = mods.join(".dune-managed.json");
    let mut managed: std::collections::BTreeMap<String, String> = std::fs::read(&lock)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    for &slug in wanted {
        match resolve(slug, minecraft, fetch) {
            Ok((name, url, sha)) => {
                let target = mods.join(&name);
                let current = sha512_of_file(&target).as_deref() == Some(sha.as_str());
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
                verified.insert(slug.to_owned(), (name.clone(), sha.clone()));
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
        .map_err(|e| format!("{}: {e}", lock.display()))?;
    if verified.len() == wanted.len() {
        write_verified(&mods, minecraft, &verified);
    }
    Ok(())
}

/// How long a successful Modrinth resolution is trusted before the next launch asks again.
const VERIFIED_TTL_SECS: u64 = 12 * 3600;
const VERIFIED_FILE: &str = ".dune-verified.json";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn write_verified(
    mods: &Path,
    minecraft: &str,
    done: &std::collections::BTreeMap<String, (String, String)>,
) {
    let stamp = serde_json::json!({ "minecraft": minecraft, "at": now_secs(), "mods": done });
    let _ = std::fs::write(mods.join(VERIFIED_FILE), serde_json::to_vec(&stamp).unwrap_or_default());
}

/// True when a fresh stamp lists every wanted slug for `minecraft` and each file is on disk with the stamped sha512.
fn verified_recently(mods: &Path, minecraft: &str, wanted: &[&str]) -> bool {
    let Ok(bytes) = std::fs::read(mods.join(VERIFIED_FILE)) else {
        return false;
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let at = v["at"].as_u64().unwrap_or(0);
    let now = now_secs();
    if v["minecraft"].as_str() != Some(minecraft) || at > now || now - at > VERIFIED_TTL_SECS {
        return false;
    }
    wanted.iter().all(|slug| {
        let entry = &v["mods"][*slug];
        match (entry[0].as_str(), entry[1].as_str()) {
            (Some(name), Some(sha)) if !name.contains(['/', '\\']) => {
                sha512_of_file(&mods.join(name)).as_deref() == Some(sha)
            }
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_fabric_range() {
        for v in ["1.8.9", "1.9.4", "1.10.2", "1.11.2", "1.12.2"] {
            assert!(is_legacy_fabric(v), "{v}");
        }
        for v in ["1.16.5", "1.18.2", "1.21.11", "26.3", "1.21"] {
            assert!(!is_legacy_fabric(v), "{v}");
        }
    }

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
    fn recent_verification_skips_modrinth_until_stale_or_changed() {
        let dir = tempfile::tempdir().unwrap();
        let mods = dir.path();
        std::fs::write(mods.join("fabric-api.jar"), b"jar").unwrap();
        let mut done = std::collections::BTreeMap::new();
        done.insert("fabric-api".to_owned(), ("fabric-api.jar".to_owned(), sha512_hex(b"jar")));
        assert!(!verified_recently(mods, "1.21.11", &["fabric-api"]), "no stamp yet");
        write_verified(mods, "1.21.11", &done);
        assert!(verified_recently(mods, "1.21.11", &["fabric-api"]));
        assert!(!verified_recently(mods, "1.21.10", &["fabric-api"]), "other minecraft version");
        assert!(!verified_recently(mods, "1.21.11", &["fabric-api", "malilib"]), "a wanted mod is missing");
        std::fs::write(mods.join("fabric-api.jar"), b"tampered").unwrap();
        assert!(!verified_recently(mods, "1.21.11", &["fabric-api"]), "file changed on disk");
        std::fs::write(mods.join("fabric-api.jar"), b"jar").unwrap();
        let stale = serde_json::json!({ "minecraft": "1.21.11", "at": now_secs() - VERIFIED_TTL_SECS - 5, "mods": done });
        std::fs::write(mods.join(VERIFIED_FILE), serde_json::to_vec(&stale).unwrap()).unwrap();
        assert!(!verified_recently(mods, "1.21.11", &["fabric-api"]), "stamp older than the TTL");
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
