//! Dune Client's own launcher update feed (NOTICE-DUNE.md). New code written for Dune Client.
//!
//! The launcher reads `https://duneclient.com/downloads/launcher-rs/latest.json`
//! (`DUNE_UPDATE_FEED` overrides it, for QA):
//!
//! ```json
//! { "version": "0.3.0",
//!   "platforms": {
//!     "linux-x86_64":   { "url": "https://…/DuneClient-0.3.0-x86_64.AppImage", "sha512": "<hex>", "sig": "<base64>" },
//!     "windows-x86_64": { "url": "https://…/DuneClient-0.3.0-windows-x64.zip",   "sha512": "<hex>", "sig": "<base64>" } } }
//! ```
//!
//! `sig` is an Ed25519 signature (base64) over [`message`]: version, platform, sha512 and url, so
//! a feed can neither swap a file nor serve an old one as new. The public key is compiled in
//! from `launcher/dune-update-key.pub`; the private one lives only on the release machine
//! (`scripts/dune-sign-update.py`, docs/ACTUALIZACIONES.md). A download is kept only when its
//! SHA-512 matches the signed one.

use ring::digest::{Context, SHA256, SHA512};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

/// The compiled-in update key(s), one base64 Ed25519 public key per line.
const KEYS: &str = include_str!("../dune-update-key.pub");

pub const DEFAULT_FEED: &str = "https://duneclient.com/downloads/launcher-rs/latest.json";

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    pub url: String,
    pub sha512: String,
    pub sig: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Feed {
    pub version: String,
    pub platforms: BTreeMap<String, Entry>,
}

pub fn feed_url() -> String {
    std::env::var("DUNE_UPDATE_FEED")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_FEED.into())
}

/// The feed's key for this build: `linux-x86_64`, `windows-x86_64`…
/// On Linux, outside an AppImage, it is `linux-x86_64-tar` (a `.tar.gz` holding `dune-launcher`).
pub fn platform() -> String {
    let base = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    if cfg!(target_os = "linux") && crate::backend::appimage_path().is_none() {
        format!("{base}-tar")
    } else {
        base
    }
}

/// What a signature covers.
pub fn message(version: &str, platform: &str, sha512: &str, url: &str) -> Vec<u8> {
    format!("dune-launcher-rs update\n{version}\n{platform}\n{sha512}\n{url}").into_bytes()
}

fn trusted_keys() -> Vec<Vec<u8>> {
    use base64::Engine;
    KEYS.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| base64::engine::general_purpose::STANDARD.decode(l).ok())
        .collect()
}

/// Checks `entry`'s signature against `keys`.
pub fn verify_with(
    feed: &Feed,
    platform: &str,
    entry: &Entry,
    keys: &[Vec<u8>],
) -> Result<(), String> {
    use base64::Engine;
    if keys.is_empty() {
        return Err("This build has no update key built in: updates are refused.".into());
    }
    let sig = base64::engine::general_purpose::STANDARD
        .decode(entry.sig.trim())
        .map_err(|_| "The update's signature is not base64.".to_string())?;
    if entry.sha512.len() != 128 || !entry.sha512.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("The update's SHA-512 is malformed.".into());
    }
    let msg = message(&feed.version, platform, &entry.sha512.to_ascii_lowercase(), &entry.url);
    let ok = keys
        .iter()
        .any(|k| UnparsedPublicKey::new(&ED25519, k).verify(&msg, &sig).is_ok());
    if ok {
        Ok(())
    } else {
        Err(format!(
            "The signature of Dune Client {} does not match the update key, so it was not installed.",
            feed.version
        ))
    }
}

/// Parses a feed and returns its entry for `platform`, signature checked.
pub fn entry_in(text: &[u8], platform: &str, keys: &[Vec<u8>]) -> Result<Option<(Feed, Entry)>, String> {
    let feed: Feed = serde_json::from_slice(text).map_err(|e| format!("latest.json: {e}"))?;
    let Some(entry) = feed.platforms.get(platform).cloned() else {
        return Ok(None);
    };
    if !entry.url.starts_with("https://") && !entry.url.starts_with("http://127.0.0.1") {
        return Err("The update's address is not https.".into());
    }
    verify_with(&feed, platform, &entry, keys)?;
    Ok(Some((feed, entry)))
}

/// `a > b` for dotted numeric versions (`0.10.0` > `0.9.2`); a pre-release suffix is ignored.
pub fn newer(a: &str, b: &str) -> bool {
    let nums = |v: &str| -> Vec<u64> {
        v.split(['.', '-', '+'])
            .map_while(|p| p.parse().ok())
            .collect()
    };
    let (x, y) = (nums(a), nums(b));
    for i in 0..x.len().max(y.len()) {
        let (p, q) = (x.get(i).copied().unwrap_or(0), y.get(i).copied().unwrap_or(0));
        if p != q {
            return p > q;
        }
    }
    false
}

/// The verified entry of the feed when it names a version newer than `own`.
pub fn check(
    fetch: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    own: &str,
) -> Result<Option<(Feed, Entry)>, String> {
    let body = fetch(&feed_url())?;
    Ok(entry_in(&body, &platform(), &trusted_keys())?.filter(|(f, _)| newer(&f.version, own)))
}

/// The launcher release the feed offers now, as the update UI knows releases (tag
/// `launcher@<version>`); None when up to date, or the feed can't be read or verified (an
/// unverifiable feed is never offered).
pub fn latest() -> Option<crate::backend::Release> {
    let (feed, entry) = check(&crate::backend::get_bytes, crate::backend::VERSION)
        .map_err(|e| eprintln!("Dune Client: update feed: {e}"))
        .ok()??;
    remember(&feed, &entry);
    Some(crate::backend::Release {
        tag: format!("{}{}", crate::backend::LAUNCHER_TAG_PREFIX, feed.version),
        name: format!("Dune Client {}", feed.version),
        body: String::new(),
        date: String::new(),
        time: String::new(),
        asset: Some((entry.url.clone(), 0)),
        prerelease: false,
        nightly: None,
    })
}

static OFFERED: std::sync::Mutex<Option<(String, Entry)>> = std::sync::Mutex::new(None);

fn remember(feed: &Feed, entry: &Entry) {
    if let Ok(mut o) = OFFERED.lock() {
        *o = Some((feed.version.clone(), entry.clone()));
    }
}

/// The verified entry offered for `version` (what [`latest`] returned).
pub fn offered(version: &str) -> Option<Entry> {
    OFFERED
        .lock()
        .ok()?
        .as_ref()
        .filter(|(v, _)| v == version)
        .map(|(_, e)| e.clone())
}

/// Downloads `entry` into `to`, keeping it only when its SHA-512 is the signed one; returns the
/// file's SHA-256 (what `backend::place_checked` checks again just before installing).
pub fn download(entry: &Entry, to: &Path, progress: &impl Fn(f32)) -> Result<String, String> {
    let response = crate::backend::get(&entry.url)?;
    let size: u64 = response
        .header("Content-Length")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    let mut reader = response.into_reader();
    let mut temp = to.as_os_str().to_owned();
    temp.push(format!(".part-{}", std::process::id()));
    let temp = std::path::PathBuf::from(temp);
    let result = (|| {
        let mut file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
        let (mut s512, mut s256) = (Context::new(&SHA512), Context::new(&SHA256));
        let (mut done, mut buf) = (0u64, vec![0u8; 1 << 16]);
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| format!("Download interrupted: {e}"))?;
            if n == 0 {
                break;
            }
            s512.update(&buf[..n]);
            s256.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            done += n as u64;
            progress((done as f32 / size.max(1) as f32).min(1.0) * 0.9);
        }
        let hex = |d: &[u8]| d.iter().map(|b| format!("{b:02x}")).collect::<String>();
        if hex(s512.finish().as_ref()) != entry.sha512.to_ascii_lowercase() {
            return Err("The download does not match the signed SHA-512: it was not installed.".into());
        }
        let sha256 = hex(s256.finish().as_ref());
        std::fs::rename(&temp, to).map_err(|e| e.to_string())?;
        Ok(sha256)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn pair() -> (Ed25519KeyPair, Vec<u8>) {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let kp = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let public = kp.public_key().as_ref().to_vec();
        (kp, public)
    }

    fn feed_json(kp: &Ed25519KeyPair, version: &str, tamper: bool) -> String {
        let sha = "ab".repeat(64);
        let url = "https://duneclient.com/downloads/launcher-rs/x.AppImage";
        let sig = base64::engine::general_purpose::STANDARD
            .encode(kp.sign(&message(version, "linux-x86_64", &sha, url)).as_ref());
        let version = if tamper { "9.9.9" } else { version };
        format!(
            r#"{{"version":"{version}","platforms":{{"linux-x86_64":{{"url":"{url}","sha512":"{sha}","sig":"{sig}"}}}}}}"#
        )
    }

    #[test]
    fn a_signed_feed_entry_is_accepted() {
        let (kp, public) = pair();
        let text = feed_json(&kp, "0.3.0", false);
        let got = entry_in(text.as_bytes(), "linux-x86_64", &[public]).unwrap();
        assert_eq!(got.unwrap().0.version, "0.3.0");
    }

    #[test]
    fn a_changed_version_or_wrong_key_is_refused() {
        let (kp, public) = pair();
        let tampered = feed_json(&kp, "0.3.0", true);
        assert!(entry_in(tampered.as_bytes(), "linux-x86_64", &[public.clone()]).is_err());
        let (_, other) = pair();
        let text = feed_json(&kp, "0.3.0", false);
        assert!(entry_in(text.as_bytes(), "linux-x86_64", &[other]).is_err());
        assert!(entry_in(text.as_bytes(), "linux-x86_64", &[]).is_err());
    }

    #[test]
    fn a_platform_the_feed_lacks_offers_nothing() {
        let (kp, public) = pair();
        let text = feed_json(&kp, "0.3.0", false);
        assert!(entry_in(text.as_bytes(), "windows-x86_64", &[public]).unwrap().is_none());
    }

    #[test]
    fn versions_compare_numerically() {
        assert!(newer("0.10.0", "0.9.2"));
        assert!(newer("1.0", "0.99.9"));
        assert!(!newer("0.3.0", "0.3.0"));
        assert!(!newer("0.2.9", "0.3.0"));
    }

    #[test]
    fn the_compiled_in_key_is_a_valid_ed25519_key() {
        let keys = trusted_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].len(), 32);
    }
}
