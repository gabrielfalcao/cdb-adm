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


adm status --all --path > "adm.status.pre.log"
sudo launchctl print system > launchctl-print-system-enabled.pre.log
sudo launchctl print-disabled system > launchctl-print-system-disabled.pre.log
sudo launchctl print user/501 > launchctl-print-user-501-enabled.pre.log
sudo launchctl print-disabled user/501 > launchctl-print-user-501-disabled.pre.log
sudo launchctl print gui/501 > launchctl-print-gui-501-enabled.pre.log
sudo launchctl print-disabled gui/501 > launchctl-print-gui-501-disabled.pre.log

rm -rf ./var
for log_name in system.log alf.log fsck_apfs.log fsck_apfs_error.log fsck_hfs.log install.log kernel-shutdown.log shutdown_monitor.log; do
    timestamp="${log_name/.log/}.pre-turn-off-$(date +"%Y%m%d-%H%M%S").log"
    target="./var/log/${log_name/.log/}_${timestamp}.log"
    mkdir -p "$(dirname "$target")"
    dd if="/var/log/${log_name}" of="${target}"
done

adm_turnoff_err_log="adm.turn-off-$(date +"%Y%m%d-%H%M%S").stderr.log"
adm_turnoff_out_log="adm.turn-off-$(date +"%Y%m%d-%H%M%S").stdout.log"
adm turn-off -viu | tee "$adm_turnoff_out_log"

for log_name in system.log alf.log fsck_apfs.log fsck_apfs_error.log fsck_hfs.log install.log kernel-shutdown.log shutdown_monitor.log; do
    timestamp="${log_name/.log/}.post-turn-off-$(date +"%Y%m%d-%H%M%S").log"
    target="./var/log/${log_name/.log/}_${timestamp}.log"
    mkdir -p "$(dirname "$target")"
    dd if="/var/log/${log_name}" of="${target}"
done

launchd_log_backup_post_turn_off="launchd.post-turn-off-$(date +"%Y%m%d-%H%M%S").log"
sudo dd if=/private/var/log/com.apple.xpc.launchd/launchd.log of="$launchd_log_backup_post_turn_off"
sudo chmod 644 "$launchd_log_backup_post_turn_off"
sudo chown $USER "$launchd_log_backup_post_turn_off"

adm status --all --path > "adm.status.post.log"
sudo launchctl print system > launchctl-print-system-enabled.post.log
sudo launchctl print-disabled system > launchctl-print-system-disabled.post.log
sudo launchctl print user/501 > launchctl-print-user-501-enabled.post.log
sudo launchctl print-disabled user/501 > launchctl-print-user-501-disabled.post.log
sudo launchctl print gui/501 > launchctl-print-gui-501-enabled.post.log
sudo launchctl print-disabled gui/501 > launchctl-print-gui-501-disabled.post.log
