#!/usr/bin/env bash
set -e
cargo check -q
cargo install -q --offline --path .
cdb export -o "cdb-export-$(date +"%Y%m%d-%H%M%S").json"
2>/dev/random 1>/dev/random cdb fix


launchd_log_backup_pre_turn_off="launchd.pre-turn-off-$(date +"%Y%m%d-%H%M%S").log"
sudo dd if=/private/var/log/com.apple.xpc.launchd/launchd.log of="$launchd_log_backup_pre_turn_off"
sudo chmod 644 "$launchd_log_backup_pre_turn_off"
sudo chown $USER "$launchd_log_backup_pre_turn_off"


adm status --all --path > "adm.status.0.log"
sudo launchctl print system > launchctl-print-system-enabled.0.log
sudo launchctl print-disabled system > launchctl-print-system-disabled.0.log
sudo launchctl print user/501 > launchctl-print-user-501-enabled.0.log
sudo launchctl print-disabled user/501 > launchctl-print-user-501-disabled.0.log
sudo launchctl print gui/501 > launchctl-print-gui-501-enabled.0.log
sudo launchctl print-disabled gui/501 > launchctl-print-gui-501-disabled.0.log

adm turn-off -viu

adm status --all --path > "adm.status.1.log"
sudo launchctl print system > launchctl-print-system-enabled.1.log
sudo launchctl print-disabled system > launchctl-print-system-disabled.1.log
sudo launchctl print user/501 > launchctl-print-user-501-enabled.1.log
sudo launchctl print-disabled user/501 > launchctl-print-user-501-disabled.1.log
sudo launchctl print gui/501 > launchctl-print-gui-501-enabled.1.log
sudo launchctl print-disabled gui/501 > launchctl-print-gui-501-disabled.1.log
