#!/usr/bin/env bash
# Creates the four Dune Client instances (Fabric + the pinned Dune Client mod) in the
# launcher data folders given by HORIZON_LAUNCHER_HOME / HORIZON_HOME.
# Build the launcher first:  cd launcher && cargo build --release -j2
set -euo pipefail
B="${1:-launcher/target/release/horizon-launcher}"
for v in 1.21.11 1.21.4 1.21.1 1.20.1; do
  "$B" instance create "Dune $v" --minecraft "$v" --loader fabric --dune
done
