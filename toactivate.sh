#!/usr/bin/env bash
set -e
cargo cbt
cargo install --offline --path .

diff -u --color adm.status.pre.log adm.status.post.log |rg '^[+]\S+\s+0' | bat -l diff | sed 's/^[+]//g' | awk '{ print $1 }' | xargs adm boot-up
