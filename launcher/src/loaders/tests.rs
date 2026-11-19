//! Lists and profiles shaped like Fabric's and Quilt's, served by a fetch that never reaches the
//! network.
use super::*;
use crate::sources::sha1_hex;
use std::collections::HashMap;
use std::fs;

#[test]
fn versions_name_the_stable_ones() {
    let fabric = r#"[{"loader": {"version": "0.17.0-beta.1", "stable": false}},
                     {"loader": {"version": "0.16.9", "stable": true}}]"#;
    let listed = parse_versions(fabric).unwrap();
    assert_eq!(pick(Kind::Fabric, &listed, None).unwrap(), "0.16.9");
    assert_eq!(
        pick(Kind::Fabric, &listed, Some("0.17.0-beta.1")).unwrap(),
        "0.17.0-beta.1"
    );
    assert!(pick(Kind::Fabric, &listed, Some("0.1"))
        .unwrap_err()
        .contains("Fabric has no loader 0.1"));
    // Quilt's comes in no order and says nothing of stability: a beta has a `-`.
    let quilt = [
        "0.20.0-beta.9",
        "0.24.0",
        "0.20.0-beta.10",
        "0.9.0",
        "0.20.0",
        "0.30.1-beta.1",
    ]
    .map(|v| format!(r#"{{"loader": {{"version": "{v}"}}}}"#))
    .join(",");
    let listed = parse_versions(&format!("[{quilt}]")).unwrap();
    let order: Vec<&str> = listed.iter().map(|l| l.version.as_str()).collect();
    assert_eq!(
        order,
        [
            "0.30.1-beta.1",
            "0.24.0",
            "0.20.0",
            "0.20.0-beta.10",
            "0.20.0-beta.9",
            "0.9.0"
        ]
    );
    assert_eq!(
        pick(Kind::Quilt, &listed, Some("latest")).unwrap(),
        "0.24.0"
    );
    assert_eq!(
        Kind::Quilt.profile_id("0.19.2", "1.21.4"),
        "quilt-loader-0.19.2-1.21.4"
    );
    assert_eq!(Kind::parse(" Fabric ").unwrap(), Kind::Fabric);
    assert_eq!(Kind::parse("NeoForge").unwrap(), Kind::NeoForge);
    assert!(Kind::parse("liteloader").is_err());
}

#[test]
fn maven_names_are_paths() {
    assert_eq!(
        maven_path("net.fabricmc:fabric-loader:0.16.9").unwrap(),
        "net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar"
    );
    assert_eq!(
        maven_path("org.lwjgl:lwjgl:3.3.3:natives-macos@zip").unwrap(),
        "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-macos.zip"
    );
    assert!(maven_path("just-a-name").is_err());
}

#[test]
fn a_profile_comes_with_its_libraries_checked() {
    let dir = std::env::temp_dir().join(format!("horizon-loaders-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let (loader_jar, asm_jar) = (b"loader".to_vec(), b"asm".to_vec());
    let profile = json!({
        "id": "fabric-loader-0.16.9-1.21.4", "inheritsFrom": "1.21.4",
        "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
        "libraries": [
            {"name": "net.fabricmc:fabric-loader:0.16.9", "url": "https://maven.fabricmc.net/",
             "sha1": sha1_hex(&loader_jar)},
            {"name": "org.ow2.asm:asm:9.7", "url": "https://maven.fabricmc.net"}
        ]
    });
    let asm = "https://maven.fabricmc.net/org/ow2/asm/asm/9.7/asm-9.7.jar";
    let mut served: HashMap<String, Vec<u8>> = HashMap::from([
        (
            "https://meta.fabricmc.net/v2/versions/loader/1.21.4/0.16.9/profile/json".into(),
            profile.to_string().into_bytes(),
        ),
        (
            "https://maven.fabricmc.net/net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar"
                .into(),
            loader_jar,
        ),
        (asm.into(), asm_jar.clone()),
        (
            format!("{asm}.sha1"),
            format!("{}  asm-9.7.jar", sha1_hex(&asm_jar)).into_bytes(),
        ),
    ]);
    let loader = Loader {
        kind: Kind::Fabric,
        version: "0.16.9".into(),
    };
    let install_with = |served: &HashMap<String, Vec<u8>>, minecraft: &str| {
        let fetch = |url: &str| served.get(url).cloned().ok_or(format!("404 {url}"));
        install(&loader, minecraft, &dir, Path::new("java"), &fetch, &|_| {})
    };
    let id = install_with(&served, "1.21.4").unwrap();
    assert_eq!(id, Kind::Fabric.profile_id("0.16.9", "1.21.4"));
    let kept: Value =
        serde_json::from_slice(&fs::read(dir.join(format!("versions/{id}/{id}.json"))).unwrap())
            .unwrap();
    assert_eq!(
        kept["libraries"][1]["downloads"]["artifact"]["path"],
        "org/ow2/asm/asm/9.7/asm-9.7.jar"
    );
    assert_eq!(
        fs::read(dir.join("libraries/org/ow2/asm/asm/9.7/asm-9.7.jar")).unwrap(),
        b"asm"
    );
    // A profile for another version, or a file that isn't the one listed, is refused.
    served.insert(
        "https://meta.fabricmc.net/v2/versions/loader/1.20.1/0.16.9/profile/json".into(),
        profile.to_string().into_bytes(),
    );
    assert!(install_with(&served, "1.20.1")
        .unwrap_err()
        .contains("not Minecraft 1.20.1"));
    fs::remove_file(dir.join("libraries/org/ow2/asm/asm/9.7/asm-9.7.jar")).unwrap();
    served.insert(asm.into(), b"tampered".to_vec());
    assert!(install_with(&served, "1.21.4")
        .unwrap_err()
        .contains("asm-9.7.jar is not the one"));
    // A repository that publishes no `.sha1` for one artifact (Legacy Fabric's lwjgl natives) still installs it.
    served.insert(asm.into(), asm_jar.clone());
    served.remove(&format!("{asm}.sha1"));
    install_with(&served, "1.21.4").unwrap();
    let kept: Value =
        serde_json::from_slice(&fs::read(dir.join(format!("versions/{id}/{id}.json"))).unwrap())
            .unwrap();
    assert_eq!(kept["libraries"][1]["downloads"]["artifact"]["sha1"], sha1_hex(&asm_jar));
    fs::remove_dir_all(&dir).unwrap();
}

/// Legacy Fabric's lwjgl-platform has no jar, only `natives-<os>` classifiers named by its `natives` map.
#[test]
fn a_library_with_only_natives_installs_its_classifiers() {
    let dir = std::env::temp_dir().join(format!("horizon-loaders-natives-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let profile = json!({
        "id": "fabric-loader-0.18.0-1.8.9", "inheritsFrom": "1.8.9", "mainClass": "x",
        "libraries": [{"name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.4+legacyfabric.17", "url": "https://maven.legacyfabric.net/",
            "natives": {"linux": "natives-linux", "osx": "natives-osx", "windows": "natives-windows"}}]
    });
    let base = "https://maven.legacyfabric.net/org/lwjgl/lwjgl/lwjgl-platform/2.9.4+legacyfabric.17/lwjgl-platform-2.9.4+legacyfabric.17";
    let mut served: HashMap<String, Vec<u8>> = HashMap::from([(
        "https://meta.legacyfabric.net/v2/versions/loader/1.8.9/0.18.0/profile/json".into(),
        profile.to_string().into_bytes(),
    )]);
    for os in ["linux", "osx", "windows"] {
        let jar = format!("jar-{os}").into_bytes();
        served.insert(format!("{base}-natives-{os}.jar.sha1"), sha1_hex(&jar).into_bytes());
        served.insert(format!("{base}-natives-{os}.jar"), jar);
    }
    let loader = Loader { kind: Kind::Fabric, version: "0.18.0".into() };
    let fetch = |url: &str| served.get(url).cloned().ok_or(format!("404 {url}"));
    let id = install(&loader, "1.8.9", &dir, Path::new("java"), &fetch, &|_| {}).unwrap();
    let kept: Value = serde_json::from_slice(&fs::read(dir.join(format!("versions/{id}/{id}.json"))).unwrap()).unwrap();
    let os = crate::java_launch::os_name();
    let key = format!("natives-{os}");
    let c = &kept["libraries"][0]["downloads"]["classifiers"][key.as_str()];
    assert!(kept["libraries"][0]["downloads"]["artifact"].is_null());
    assert_eq!(c["sha1"], sha1_hex(format!("jar-{os}").as_bytes()));
    assert!(dir.join("libraries").join(c["path"].as_str().unwrap()).is_file());
    assert!(crate::backend::loader_profile_usable(&dir.join(format!("versions/{id}/{id}.json"))));
    fs::remove_dir_all(&dir).unwrap();
}

/// The real services and repositories (network): `cargo test loaders -- --ignored`.
#[test]
#[ignore]
fn the_real_loaders_install() {
    for kind in ["fabric", "quilt"] {
        let dir =
            std::env::temp_dir().join(format!("horizon-loaders-{kind}-{}", std::process::id()));
        let loader = choose(kind, "1.21.4", None).unwrap();
        let id = install(
            &loader,
            "1.21.4",
            &dir,
            Path::new("java"),
            &crate::backend::get_bytes,
            &|l| eprintln!("{l}"),
        )
        .unwrap();
        assert_eq!(id, loader.kind.profile_id(&loader.version, "1.21.4"));
        let kept: Value = serde_json::from_slice(
            &fs::read(dir.join(format!("versions/{id}/{id}.json"))).unwrap(),
        )
        .unwrap();
        for library in kept["libraries"].as_array().unwrap() {
            let path = library["downloads"]["artifact"]["path"].as_str().unwrap();
            assert!(dir.join("libraries").join(path).is_file(), "{path}");
        }
        fs::remove_dir_all(&dir).unwrap();
    }
}
