#!/usr/bin/env bash
set -ex
cargo cbt
cargo install --offline --path .
# cdb export -o "cdb-export-$(date +"%Y%m%d-%H%M%S").json"
# cdb fix
timestamp="$(t16g)"
rm -f turn-off.*.*.log
stderr_log="turn-off.${timestamp}.err.log"
stdout_log="turn-off.${timestamp}.out.log"
1>$stdout_log 2>$stderr_log sudo adm turn-off -vi
