use std::collections::BTreeMap;
use std::process::{Command, Stdio};

use crate::{CDBSettings, Error, Result};

pub fn export_domain(domain: impl std::fmt::Display) -> Result<plist::Value> {
    let domain = domain.to_string();
    let path = iocore::Path::tmp_file();
    let (exit_code, _, err) = defaults(&["export", &domain.to_string(), &path.to_string()])?;
    if exit_code != 0 {
        return Err(Error::IOError(format!(
            "defaults export {} failed[{}]: {}",
            &domain, exit_code, err
        )));
    }
    let bytes = match path.read_bytes() {
        Ok(bytes) => {
            path.delete_unchecked();
            bytes
        },
        Err(e) => {
            path.delete_unchecked();
            return Err(e.into());
        },
    };
    let plist = plist::from_bytes::<plist::Value>(&bytes)?;
    Ok(plist)
}

pub fn defaults_delete_domain(domain: impl std::fmt::Display) -> Result<(String, plist::Value)> {
    let domain = domain.to_string();
    let plist = export_domain(&domain)?;
    let shell_result = defaults(&["delete", &domain.to_string()])?;
    match shell_result {
        (0, _, _) => Ok((domain, plist)),
        (exit_code, _, err) => Err(Error::IOError(format!(
            "defaults delete {} failed[{}]: {}",
            &domain, exit_code, err
        ))),
    }
}
pub fn delete_domains(domains: &[&str]) -> Result<DeleteDefaultsMacOSResult> {
    let mut errors = BTreeMap::<String, Error>::new();
    let mut domain_map = export_domains(domains, true)?;

    for domain in domains {
        match defaults_delete_domain(&domain) {
            Ok((domain, plist)) => {
                domain_map.insert(domain.to_string(), plist);
            },
            Err(e) => {
                errors.insert(domain.to_string(), e);
            },
        }
    }
    Ok(DeleteDefaultsMacOSResult { domain_map, errors })
}

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteDefaultsMacOSResult {
    pub domain_map: BTreeMap<String, plist::Value>,
    pub errors: BTreeMap<String, Error>,
}
pub fn list_domains() -> Result<Vec<String>> {
    match defaults(&["domains"])? {
        (0, csv, _) =>
            Ok(csv.split(",").map(|domain| domain.trim().to_string()).collect::<Vec<String>>()),
        (exit_code, _, error_message) => Err(Error::IOError(format!(
            "command `default domains' exited[{}]: {}",
            exit_code, error_message
        ))),
    }
}
pub fn export_domains(domains: &[&str], global: bool) -> Result<BTreeMap<String, plist::Value>> {
    let mut data = BTreeMap::<String, plist::Value>::new();
    if global {
        data.insert("NSGlobalDomain".to_string(), export_domain("NSGlobalDomain")?);
    }
    for domain in domains {
        data.insert(domain.to_string(), export_domain(&domain)?);
    }
    Ok(data)
}
pub fn export_plists_from_path(path: &str) -> Result<BTreeMap<String, plist::Value>> {
    let path_domains =
        iocore::walk_dir(iocore::Path::raw(path), iocore::NoopProgressHandler, None)?
            .iter()
            .filter(|path| {
                path.is_file() && path.extension().unwrap_or_default().ends_with("plist")
            })
            .map(|path| path.to_string())
            .collect::<Vec<String>>();

    Ok(export_domains(
        &path_domains.iter().map(|domain| domain.as_str()).collect::<Vec<&str>>(),
        false,
    )?)
}
pub fn export_library_preferences() -> Result<BTreeMap<String, plist::Value>> {
    Ok(export_plists_from_path("/Library/Preferences")?)
}
pub fn export_all_domains() -> Result<BTreeMap<String, plist::Value>> {
    Ok(export_domains(
        &list_domains()?
            .iter()
            .filter(|domain| !domain.is_empty())
            .map(|domain| domain.as_str())
            .collect::<Vec<&str>>(),
        true,
    )?)
}
pub fn defaults(args: &[&str]) -> Result<(i64, String, String)> {
    let (exit_code, stdout, stderr) = defaults_ok(args)?;
    if exit_code == 0 {
        Ok((exit_code, stdout, stderr))
    } else {
        let command = format!("defaults {}", args.join(" "));
        Err(Error::IOError(format!(
            "command `{}' failed with exit code {}",
            command, exit_code
        )))
    }
}
pub fn defaults_ok(args: &[&str]) -> Result<(i64, String, String)> {
    let mut cmd = Command::new("defaults");
    let cmd = cmd.current_dir("/System");
    let cmd = cmd.args(args);
    let cmd = cmd.stdin(Stdio::null());
    let cmd = cmd.stdout(Stdio::piped());
    let cmd = cmd.stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;
    let exit_code: i64 = output.status.code().unwrap_or_default().into();
    Ok((
        exit_code,
        String::from_utf8(output.stdout).unwrap_or_default(),
        String::from_utf8(output.stderr).unwrap_or_default(),
    ))
}

pub fn coredata_fix(quiet: bool) -> Result<()> {
    let settings = CDBSettings::cli(quiet);

    for args in settings.defaults_exec_args() {
        defaults_ok(
            &args
                .iter()
                .filter(|domain| !domain.is_empty())
                .map(|domain| domain.as_str())
                .collect::<Vec<&str>>(),
        )?;
        if !quiet {
            eprintln!("defaults {} -", args.join(" "));
        }
    }
    for args in defaults_exec_args() {
        defaults_ok(&args)?;
        if !quiet {
            eprintln!("defaults {} -", args.join(" "));
        }
    }
    Ok(())
}
fn defaults_exec_args<'a>() -> Vec<Vec<&'a str>> {
    use iocore::Path;
    let screencapture = Path::new("~").to_string();
    vec![
        vec!["delete", "NSGlobalDomain", "NSLinguisticDataAssetsRequested"],
        vec!["delete", "NSGlobalDomain", "NSPreferredWebServices"],
        vec!["delete", "NSGlobalDomain", "AppleInterfaceStyle"],
        vec![
            "delete",
            "NSGlobalDomain",
            "com.apple.gms.availability.useCasesWhoseAssetsNotReady",
        ],
        vec!["delete", "NSGlobalDomain", "com.apple.gms.availability.disallowedUseCases"],
        vec!["delete", "com.pixelmatorteam.pixelmator.x"],
        vec!["delete", "sharedfilelistd"],
        vec!["delete", "com.qtproject"],
        vec!["delete", "com.apple.TV"],
        vec!["delete", "com.apple.universalaccessAuthWarning"],
        vec!["delete", "ZoomChat"],
        vec!["delete", "com.Syncrosoft.LCC"],
        vec!["delete", "com.apple.AMPLibraryAgent"],
        vec!["delete", "com.apple.Accessibility"],
        vec!["delete", "com.apple.Accessibility-Settings.extension"],
        vec!["delete", "com.apple.Accessibility.Assets"],
        vec!["delete", "com.apple.ActivityMonitor"],
        vec!["delete", "com.apple.AdLib"],
        vec!["delete", "com.apple.AdPlatforms"],
        vec!["delete", "com.apple.AddressBook"],
        vec!["delete", "com.apple.AppleIntelligenceReport"],
        vec!["delete", "com.apple.AppleMediaServices"],
        vec!["delete", "com.apple.AppleMediaServices.notbackedup"],
        vec!["delete", "com.apple.AppleMultitouchMouse"],
        vec!["delete", "com.apple.AvatarUI.Staryu"],
        vec!["delete", "com.apple.BKAgentService"],
        vec!["delete", "com.apple.BluetoothFileExchange"],
        vec!["delete", "com.apple.CallHistorySyncHelper"],
        vec!["delete", "com.apple.Chess"],
        vec!["delete", "com.apple.CloudSubscriptionFeatures.cache"],
        vec!["delete", "com.apple.CloudSubscriptionFeatures.config"],
        vec!["delete", "com.apple.CloudSubscriptionFeatures.gmCache"],
        vec!["delete", "com.apple.CloudTelemetryService.xpc"],
        vec!["delete", "com.apple.CommCenter.counts"],
        vec!["delete", "com.apple.CoreGraphics"],
        vec!["delete", "com.apple.CrashReporter"],
        vec!["delete", "com.apple.DataDeliveryServices"],
        vec!["delete", "com.apple.DiscHelper"],
        vec!["delete", "com.apple.DuetExpertCenter.AppPredictionExpert"],
        vec!["delete", "com.apple.EscrowSecurityAlert"],
        vec!["delete", "com.apple.GEO"],
        vec!["delete", "com.apple.HearingAids"],
        vec!["delete", "com.apple.IFTelemetrySELFIngestor"],
        vec!["delete", "com.apple.LaunchServices"],
        vec!["delete", "com.apple.Maps"],
        vec!["delete", "com.apple.Maps.mapssyncd"],
        vec!["delete", "com.apple.MobileSMS"],
        vec!["delete", "com.apple.Music"],
        vec!["delete", "com.apple.Notes"],
        vec!["delete", "com.apple.PersonalAudio"],
        vec!["delete", "com.apple.Preview.ViewState"],
        vec!["delete", "com.apple.ProblemReporter"],
        vec!["delete", "com.apple.QuickTimePlayerX"],
        vec!["delete", "com.apple.ReportCrash"],
        vec!["delete", "com.apple.STMExtension.Mail"],
        vec!["delete", "com.apple.SafariTechnologyPreview"],
        vec!["delete", "com.apple.ScreenTimeAgent"],
        vec!["delete", "com.apple.ServicesMenu.Services"],
        vec!["delete", "com.apple.Siri.SiriTodayExtension"],
        vec!["delete", "com.apple.SiriNCService"],
        vec!["delete", "com.apple.SpeakSelection"],
        vec!["delete", "com.apple.SpeechRecognitionCore"],
        vec!["delete", "com.apple.StorageManagement.Service"],
        vec!["delete", "com.apple.TTY"],
        vec!["delete", "com.apple.TV"],
        vec!["delete", "com.apple.TelephonyUtilities"],
        vec!["delete", "com.apple.TelephonyUtilities.sharePlayAppPolicies"],
        vec!["delete", "com.apple.TestFlight"],
        vec!["delete", "com.apple.TextEdit"],
        vec!["delete", "com.apple.UnifiedAssetFramework"],
        vec!["delete", "com.apple.VoiceMemos"],
        vec!["delete", "com.apple.VoiceOver4.local"],
        vec!["delete", "com.apple.VoiceOverUtility"],
        vec!["delete", "com.apple.Wallpaper-Settings.extension"],
        vec!["delete", "com.apple.WatchListKit"],
        vec!["delete", "com.apple.accessibility.heard"],
        vec!["delete", "com.apple.amp.mediasharingd"],
        vec!["delete", "com.apple.amsengagementd"],
        vec!["delete", "com.apple.animoji"],
        vec!["delete", "com.apple.archiveutility"],
        vec!["delete", "com.apple.assistantd"],
        vec!["delete", "com.apple.biomesyncd"],
        vec!["delete", "com.apple.bookdatastored"],
        vec!["delete", "com.apple.calaccessd"],
        vec!["delete", "com.apple.chronod"],
        vec!["delete", "com.apple.classroom"],
        vec!["delete", "com.apple.cloudd"],
        vec!["delete", "com.apple.cloudpaird"],
        vec!["delete", "com.apple.commerce.knownclients"],
        vec!["delete", "com.apple.coreservices.uiagent"],
        vec!["delete", "com.apple.corespotlightui"],
        vec!["delete", "com.apple.diskspaced"],
        vec!["delete", "com.apple.driver.AppleBluetoothMultitouch.mouse"],
        vec!["delete", "com.apple.dt.Xcode"],
        vec!["delete", "com.apple.finder"],
        vec!["delete", "com.apple.findmy"],
        vec!["delete", "com.apple.frameworks.diskimages.diuiagent"],
        vec!["delete", "com.apple.gamed"],
        vec!["delete", "com.apple.homeenergyd"],
        vec!["delete", "com.apple.iApps"],
        vec!["delete", "com.apple.iBooksX"],
        vec!["delete", "com.apple.iCal"],
        vec!["delete", "com.apple.iCloudNotificationAgent"],
        vec!["delete", "com.apple.ibtool"],
        vec!["delete", "com.apple.icloud.gm"],
        vec!["delete", "com.apple.icloud.searchpartyuseragent"],
        vec!["delete", "com.apple.iclouddrive.features"],
        vec!["delete", "com.apple.imagecapture"],
        vec!["delete", "com.apple.imdpersistence.IMDPersistenceAgent"],
        vec!["delete", "com.apple.inputAnalytics.IASGenmojiAnalyzer"],
        vec!["delete", "com.apple.inputAnalytics.IASSRAnalyzer"],
        vec!["delete", "com.apple.inputAnalytics.IASWTAnalyzer"],
        vec!["delete", "com.apple.inputmethod.Kotoeri"],
        vec!["delete", "com.apple.itunescloud.daemon"],
        vec!["delete", "com.apple.java.util.prefs"],
        vec!["delete", "com.apple.keyboardservicesd"],
        vec!["delete", "com.apple.keychainaccess"],
        vec!["delete", "com.apple.lighthouse.dill.BiomeSELFIngestor"],
        vec!["delete", "com.apple.lighthouse.siri.IFTranscriptIngestor"],
        vec!["delete", "com.apple.madrid"],
        vec!["delete", "com.apple.mail"],
        vec!["delete", "com.apple.mediaanalysisd"],
        vec!["delete", "com.apple.menuextra.textinput"],
        vec!["delete", "com.apple.mlhost"],
        vec!["delete", "com.apple.mlruntimed"],
        vec!["delete", "com.apple.mobiletimer"],
        vec!["delete", "com.apple.mobiletimerd"],
        vec!["delete", "com.apple.ncprefs"],
        vec!["delete", "com.apple.networkserviceproxy"],
        vec!["delete", "com.apple.news.tag"],
        vec!["delete", "com.apple.newscore"],
        vec!["delete", "com.apple.notificationcenterui"],
        vec!["delete", "com.apple.onetimepasscodes"],
        vec!["delete", "com.apple.photoanalysisd"],
        vec!["delete", "com.apple.photolibraryd"],
        vec!["delete", "com.apple.photos.shareddefaults"],
        vec!["delete", "com.apple.preferences.softwareupdate"],
        vec!["delete", "com.apple.print.add"],
        vec!["delete", "com.apple.proactive.PersonalizationPortrait"],
        vec!["delete", "com.apple.screencaptureui"],
        vec!["delete", "com.apple.security.cloudkeychainproxy3.keysToRegister"],
        vec!["delete", "com.apple.security.pboxd"],
        vec!["delete", "com.apple.seserviced"],
        vec!["delete", "com.apple.siri.VoiceShortcuts"],
        vec!["delete", "com.apple.siri.media-indexer"],
        vec!["delete", "com.apple.siri.morphun"],
        vec!["delete", "com.apple.siri.shortcuts"],
        vec!["delete", "com.apple.siriactionsd"],
        vec!["delete", "com.apple.siriknowledged"],
        vec!["delete", "com.apple.sociallayerd"],
        vec!["delete", "com.apple.sociallayerd.CloudKit.ckwriter"],
        vec!["delete", "com.apple.speakerrecognition"],
        vec!["delete", "com.apple.spotlightknowledge"],
        vec!["delete", "com.apple.stickersd"],
        vec!["delete", "com.apple.stocks.account"],
        vec!["delete", "com.apple.stocks.detailintents"],
        vec!["delete", "com.apple.stocks.stockskit"],
        vec!["delete", "com.apple.stocks2"],
        vec!["delete", "com.apple.suggestd"],
        vec!["delete", "com.apple.syncdefaultsd"],
        vec!["delete", "com.apple.syncserver"],
        vec!["delete", "com.apple.talagent"],
        vec!["delete", "com.apple.timemachine.HelperAgent"],
        vec!["delete", "com.apple.tipsd"],
        vec!["delete", "com.apple.translationd"],
        vec!["delete", "com.apple.transparencyd"],
        vec!["delete", "com.apple.universalaccess"],
        vec!["delete", "com.apple.universalaccessAuthWarning"],
        vec!["delete", "com.apple.visualintelligence"],
        vec!["delete", "com.apple.voiceservices"],
        vec!["delete", "com.apple.weather.sensitive"],
        vec!["delete", "com.apple.weather.widget"],
        vec!["delete", "com.google.Chrome.canary"],
        vec!["delete", "com.google.Keystone.Agent"],
        vec!["delete", "com.google.chrome"],
        vec!["delete", "com.google.chrome.for.testing"],
        vec!["delete", "group.com.apple.photolibraryd.private"],
        vec!["delete", "org.openemu.OpenEmu"],
        vec!["delete", "pbs"],
        vec!["delete", "systemgroup.com.apple.icloud.searchpartyd.sharedsettings"],
        vec!["delete", "NSGlobaldomain", "AKLastEmailListRequestDateKey"],
        vec![
            "delete",
            "com.apple.Multitouch.preferencesBackup",
            "22F82",
            "com.apple.driver.AppleBluetoothMultitouch.trackpad",
        ],
        vec![
            "delete",
            "com.apple.Multitouch.preferencesBackup",
            "22F82",
            "com.apple.driver.AppleBluetoothMultitouch.mouse",
        ],
        vec!["write NSGlobaldomain", "AppleKeyboardUIMode", "-integer", "2"],
        vec!["write NSGlobaldomain", "AppleLanguages", "-array", "en-US"],
        vec!["write NSGlobaldomain", "AppleLocale", "-string", "en-US"],
        vec!["write com.apple.dock", "wvous-br-corner", "-bool", "NO"],
        vec!["write com.apple.dock", "showAppExposeGestureEnabled", "-bool", "NO"],
        vec!["write com.apple.dock", "show-recents", "-bool", "NO"],
        vec!["write", "com.apple.dock", "autohide", "-bool", "true"],
        vec!["write", "com.apple.dock", "autohide-time-modifier", "-float", "0.5"],
        vec![
            "write",
            "/Library/Preferences/com.apple.mDNSResponder.plist",
            "NoMulticastAdvertisements",
            "-bool",
            "YES",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.bluetooth.plist",
            "BluetoothAutoSeekKeyboard",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.bluetooth.plist",
            "BluetoothAutoSeekPointingDevice",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.bluetooth.plist",
            "SpatialSoundProfileAllowed",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.bluetooth.plist",
            "move3PPLEMSToLegacyModeSerial",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.TimeMachine.plist",
            "PreferencesVersion",
            "-integer",
            "1",
        ],
        vec!["write", "com.apple.Terminal", "StringEncodings", "-array", "4"],
        vec!["write", "com.apple.screensaver", "askForPassword", "-integer", "1"],
        vec!["write", "com.apple.screensaver", "askForPasswordDelay", "-integer", "0"],
        vec!["write", "NSGlobalDomain", "AppleLanguages", "-array", "\"en-US\""],
        vec!["write", "NSGlobalDomain", "KeyRepeat", "-integer", "1"],
        vec!["write", "NSGlobalDomain", "AppleKeyboardUIMode", "-integer", "2"],
        vec!["write", "NSGlobalDomain", "InitialKeyRepeat", "-integer", "1"],
        vec!["write", "NSGlobalDomain", "com.apple.keyboard.fnState", "-integer", "0"],
        vec![
            "delete",
            "NSGlobalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
        ],
        vec![
            "write",
            "NSGlobalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
            "-array",
            "us",
        ],
        vec!["write", "NSGlobalDomain", "NSWebServicesProviderWebSearch", "-dict", "NSWebServicesProviderWebSearch"],
        vec!["write", "NSGlobalDomain", "NSSpellCheckerContainerTransitionComplete", "-bool", "NO"],
        vec!["write", "NSGlobalDomain", "NSSpellCheckerDictionaryContainerTransitionComplete", "-bool", "NO"],
        vec!["write", "NSGlobalDomain", "NSUserQuotesArray", "-array", "\\U201c", "\\U201d", "\\U2018", "\\U2019"],
        vec![
            "write",
            "NSGlobalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
            "-dict",
            "KB_SpellingLanguage",
            "-dict",
            "KB_SpellingLanguageIsAutomatic",
            "-bool",
            "NO",
        ],
        vec!["write", "NSGlobalDomain", "AppleShowScrollBars", "-string", "Always"],
        vec!["write", "com.apple.finder", "_FXShowPosixPathInTitle", "-bool", "true"],
        vec!["write", "com.apple.finder", "ShowExternalHardDrivesOnDesktop", "-bool", "NO"],
        vec!["write", "com.apple.finder", "ShowHardDrivesOnDesktop", "-bool", "NO"],
        vec!["write", "com.apple.finder", "ShowMountedServersOnDesktop", "-bool", "NO"],
        vec!["write", "com.apple.finder", "ShowRemovableMediaOnDesktop", "-bool", "NO"],
        vec!["write", "com.apple.finder", "FXDefaultSearchScope", "-string", "SCcf"],
        vec![
            "write",
            "com.apple.screencapture",
            "location",
            "-string",
            screencapture.clone().leak(),
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.driver.AppleIRController.plist",
            "DeviceEnabled",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "activity_report_denominator_network_experiments",
            "-integer",
            "1",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "activity_report_denominator_network_speed_test",
            "-integer",
            "1000",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "disable_quic_race5",
            "-integer",
            "1",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "ech_probe_denominator",
            "-integer",
            "5000",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "ech_probe_numerator",
            "-integer",
            "5",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "ech_canary_denominator",
            "-integer",
            "5000",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "ech_canary_numerator",
            "-integer",
            "5",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "enable_quic",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "enable_unified_http",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "enable_accurate_ecn",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "ech_probe_enabled",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "enable_l4s",
            "-bool",
            "YES",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkd.plist",
            "enable_tcp_l4s",
            "-bool",
            "YES",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.networkextension.control.plist",
            "CriticalDomains",
            "-array",
            "nsa.gov",
            "darpa.mil",
        ],
        vec![
            "write",
            "/Library/Preferences/com.apple.security.appsandbox.plist",
            "UnrestrictSpotlightContainerScope",
            "-bool",
            "YES",
        ],
        vec![
            "write",
            "com.apple.SafariTechnologyPreview",
            "IncludeInternalDebugMenu",
            "-bool",
            "NO",
        ],
    ]
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use plist::{Dictionary, Value};

    use super::{export_all_domains, list_domains};
    use crate::Result;
    #[test]
    fn test_list_domains() -> Result<()> {
        let domains = list_domains()?;
        assert_eq!(domains.is_empty(), false);
        assert_eq!(domains.contains(&"com.apple.FontBook".to_string()), true);
        Ok(())
    }
    #[test]
    fn test_export_all_domains() -> Result<()> {
        let domains: BTreeMap<String, plist::Value> = export_all_domains()?;
        assert_eq!(domains.is_empty(), false);
        assert_eq!(domains.contains_key(&"com.apple.Safari".to_string()), true);
        let safari = match domains.get("com.apple.Safari").unwrap() {
            Value::Dictionary(safari) => safari.clone(),
            _ => Dictionary::new(),
        };
        let extensions_enabled = match safari.get("ExtensionsEnabled").unwrap() {
            Value::Boolean(extensions_enabled) => extensions_enabled.clone(),
            _ => false,
        };
        assert_eq!(extensions_enabled, true);
        Ok(())
    }
}
