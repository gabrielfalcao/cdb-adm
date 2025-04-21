#!/usr/bin/env bash

diff -u --color adm.status.pre.log adm.status.post.log |rg '^[+]\S+\s+0' | bat -l diff
