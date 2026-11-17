#!/usr/bin/env python3
"""Builds the update feed `latest.json` for the Dune launcher (Rust) and signs every artifact.

usage: dune-sign-update.py VERSION linux-x86_64=URL=FILE [windows-x86_64=URL=FILE ...] > latest.json

Signing key: ~/.config/dune/launcher-rs-signing.key (PEM, Ed25519, mode 0600), or $DUNE_UPDATE_KEY.
The signed message is exactly what launcher/src/dune_update.rs::message builds:
    "dune-launcher-rs update\n<version>\n<platform>\n<sha512 hex>\n<url>"
"""
import base64, hashlib, json, os, sys
from cryptography.hazmat.primitives.serialization import load_pem_private_key

key_path = os.environ.get("DUNE_UPDATE_KEY", os.path.expanduser("~/.config/dune/launcher-rs-signing.key"))
if os.stat(key_path).st_mode & 0o077:
    sys.exit(f"{key_path} must be mode 0600")
key = load_pem_private_key(open(key_path, "rb").read(), None)
version, entries = sys.argv[1], sys.argv[2:]
platforms = {}
for e in entries:
    plat, url, path = e.split("=", 2)
    sha = hashlib.sha512(open(path, "rb").read()).hexdigest()
    msg = f"dune-launcher-rs update\n{version}\n{plat}\n{sha}\n{url}".encode()
    platforms[plat] = {"url": url, "sha512": sha, "sig": base64.b64encode(key.sign(msg)).decode()}
print(json.dumps({"version": version, "platforms": platforms}, indent=2))
