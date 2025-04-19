#!/usr/bin/env bash
set -ex
cargo cbt
cargo install --offline --path .
cdb export -o "cdb-export-$(date +"%Y%m%d-%H%M%S").json"
2>/dev/random 1>/dev/random cdb fix
adm turn-off -vi
