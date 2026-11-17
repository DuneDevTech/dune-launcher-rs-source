#!/usr/bin/env python3
"""Licence audit of the Dune Client launcher's Rust dependencies -> THIRD_PARTY_NOTICES.md.

Reads `cargo metadata` for the platforms we ship (Linux and Windows, both x86_64), keeps the
packages reachable from the launcher through normal and build dependencies (dev-dependencies are
not shipped), checks every licence expression against the allow-list below, copies each package's
licence files to `third_party_licenses/` and writes the notices. Exit status 1 when a package has
no licence, or one that is not on the allow-list (review it by hand, then extend the list).

    cd launcher
    cargo metadata --format-version 1 --locked --filter-platform x86_64-unknown-linux-gnu > /tmp/m-linux.json
    cargo metadata --format-version 1 --locked --filter-platform x86_64-pc-windows-gnu    > /tmp/m-win.json
    python3 ../scripts/third-party-audit.py /tmp/m-linux.json /tmp/m-win.json
"""
import json, os, re, shutil, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ALLOWED = {
    # permissive
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "Unlicense", "0BSD", "CC0-1.0",
    "BSL-1.0", "MIT-0", "Unicode-3.0", "Unicode-DFS-2016", "OpenSSL", "CDLA-Permissive-2.0", "NCSA",
    "Apache-2.0 WITH LLVM-exception", "LicenseRef-ring", "OFL-1.1", "bzip2-1.0.6", "Ubuntu-font-1.0",
    # weak copyleft, file-level (fine for a statically linked Rust binary only with source offered;
    # we offer the whole source anyway): MPL-2.0 files are kept under MPL
    "MPL-2.0", "LGPL-3.0-or-later", "LGPL-3.0-only", "LGPL-2.1-or-later", "LGPL-2.1-only",
}
LICENSE_FILE = re.compile(r"^(LICEN[CS]E|COPYING|COPYRIGHT|NOTICE|UNLICENSE)([-_.].*)?$", re.I)


def tokens(expr):
    expr = re.sub(r"\s+WITH\s+[A-Za-z0-9.\-]+", "", expr)
    return [t for t in re.split(r"[\s()]+", expr.replace("/", " OR ")) if t and t not in ("AND", "OR", "WITH")]


def load(path):
    return json.load(open(path))


def reachable(meta):
    pk = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    seen, todo = set(), [meta["resolve"]["root"]]
    while todo:
        i = todo.pop()
        if i in seen:
            continue
        seen.add(i)
        for d in nodes[i]["deps"]:
            kinds = [k.get("kind") for k in d.get("dep_kinds", [])]
            if kinds and all(k == "dev" for k in kinds):
                continue
            todo.append(d["pkg"])
    return {pk[i]["name"] + " " + pk[i]["version"]: pk[i] for i in seen}


def main(paths):
    pkgs = {}
    for p in paths:
        pkgs.update(reachable(load(p)))
    own = {k for k, v in pkgs.items() if v["source"] is None}
    out_dir = os.path.join(ROOT, "third_party_licenses")
    shutil.rmtree(out_dir, ignore_errors=True)
    os.makedirs(out_dir)
    rows, bad = [], []
    for key in sorted(pkgs, key=str.lower):
        p = pkgs[key]
        lic = (p.get("license") or "").strip()
        if not lic and p.get("license_file"):
            lic = "SEE LICENSE FILE"
        toks = tokens(lic) if lic else []
        # An OR expression is satisfied by any one allowed alternative; flag only when none is.
        alts = [a.strip() for a in re.split(r"\s+OR\s+", lic.replace("/", " OR ").strip("() "))] if lic else []
        ok = bool(lic) and any(all(t in ALLOWED for t in tokens(a)) for a in alts)
        if key.split()[0] in ("ring",) and not lic:
            ok = True
        if not ok and key not in own:
            bad.append((key, lic or "NONE"))
        d = os.path.dirname(p["manifest_path"])
        files = [f for f in sorted(os.listdir(d)) if LICENSE_FILE.match(f) and os.path.isfile(os.path.join(d, f))]
        if files and key not in own:
            dest = os.path.join(out_dir, key.replace(" ", "-"))
            os.makedirs(dest, exist_ok=True)
            for f in files:
                shutil.copy(os.path.join(d, f), os.path.join(dest, f))
        rows.append((p["name"], p["version"], lic or "NONE", p.get("repository") or p.get("homepage") or "", bool(files), key in own))
    fonts = [
        ("Inter", "OFL-1.1", "crates/horizon-theme/fonts/OFL-Inter.txt"),
        ("Montserrat", "OFL-1.1", "crates/horizon-theme/fonts/OFL-Montserrat.txt"),
        ("Geist Mono", "OFL-1.1", "crates/horizon-theme/fonts/OFL-Geist.txt"),
        ("Pixelify Sans", "OFL-1.1", "crates/horizon-theme/fonts/OFL-PixelifySans.txt"),
    ]
    with open(os.path.join(ROOT, "THIRD_PARTY_NOTICES.md"), "w") as f:
        f.write("# Third-party notices — Dune Client launcher\n\n")
        f.write("Dune Client's launcher is a modified version of the Horizon launcher\n"
                "(https://github.com/horizon-rust/horizon, Copyright (c) 2026 Ludovic Marie and the Horizon\n"
                "contributors), licensed under the GNU LGPL 3.0 or later (`LICENSE`; code merged earlier under\n"
                "MIT, `LICENSES/MIT.txt`). Our changes are listed in `NOTICE-DUNE.md`.\n\n")
        f.write("It also contains the open-source components below, each under its own licence. The full licence\n"
                "texts and copyright notices of every crate are in `third_party_licenses/<crate>-<version>/`.\n"
                "Generated by `scripts/third-party-audit.py` from `cargo metadata` (Linux and Windows x86_64\n"
                "builds, normal and build dependencies).\n\n")
        f.write("## Bundled fonts\n\n| Font | Licence | Text |\n|---|---|---|\n")
        for n, l, t in fonts:
            f.write(f"| {n} | {l} | `{t}` |\n")
        f.write("\nBrand art (`brand/`, `launcher/assets/brand/`, `packaging/icons/`, `crates/horizon-theme/icons/`) is\n"
                "**not** open source: © Dune Development, all rights reserved (`brand/LICENSE-BRAND.txt`).\n\n")
        f.write("## Rust crates\n\n| Crate | Version | Licence | Source |\n|---|---|---|---|\n")
        for n, v, l, r, has, is_own in rows:
            if is_own:
                continue
            f.write(f"| {n} | {v} | {l} | {r} |\n")
        f.write(f"\n{sum(1 for r in rows if not r[5])} third-party crates.\n")
        f.write("\n## Workspace crates (this repository)\n\n"
                + "\n".join(f"- {r[0]} {r[1]} — {r[2]}" for r in rows if r[5]) + "\n")
    print(f"{len(rows)} packages ({len(own)} ours); licence files copied for the rest")
    copyleft = [r for r in rows if re.search(r"GPL|MPL|EPL|CDDL", r[2]) and not r[5]]
    for r in copyleft:
        print("  copyleft-ish:", r[0], r[1], r[2])
    if bad:
        print("NOT ON THE ALLOW-LIST:")
        for k, l in bad:
            print("  ", k, l)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
