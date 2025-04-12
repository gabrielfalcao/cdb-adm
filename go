#!/usr/bin/env bash
set -ex
cargo cbt
cargo install --offline --path .
cdb export > cdb-export-$(date +"%Y%m%d-%H%M%S").json
cdb fix
sudo cdb fix
sudo adm turn-off -vi
