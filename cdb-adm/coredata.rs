use std::collections::BTreeMap;
use std::io::{pipe, Write};
use std::process::{Command, Stdio};

use crate::{to_slice_str, to_vec_string, CDBSettings, Error, Result};

pub fn defaults_write(domain: impl std::fmt::Display, key: &[&str]) -> Result<plist::Value> {
    validate_domain_path_for_current_user(&domain)?;
    let mut args = vec!["write".to_string(), domain.to_string()];
    args.extend(to_vec_string!(key));

    let (exit_code, out, err) = defaults(to_slice_str!(args))?;
    if exit_code != 0 {
        return Err(Error::IOError(format!(
            "defaults export {} failed[{}]: {}",
            &domain, exit_code, err
        )));
    }
    let plist = plist::from_bytes::<plist::Value>(out.as_bytes())?;
    Ok(plist)
}

pub fn export_domain(domain: impl std::fmt::Display) -> Result<plist::Value> {
    let domain = domain.to_string();
    let (exit_code, out, err) = defaults(&["export", &domain.to_string(), "-"])?;
    if exit_code != 0 {
        return Err(Error::IOError(format!(
            "defaults export {} failed[{}]: {}",
            &domain, exit_code, err
        )));
    }
    let plist = plist::from_bytes::<plist::Value>(out.as_bytes())?;
    Ok(plist)
}

pub fn defaults_delete_domain(domain: impl std::fmt::Display) -> Result<(String, plist::Value)> {
    let domain = domain.to_string();
    let plist = export_domain(&domain)?;
    let shell_result = defaults(&["delete", &domain.to_string()])?;
    match shell_result {
        (0 | 1, _, _) => Ok((domain, plist)),
        (exit_code, _, err) => Err(Error::IOError(format!(
            "defaults delete {} failed[{}]: {}",
            &domain, exit_code, err
        ))),
    }
}
pub fn defaults_delete(key: &[&str]) -> Result<()> {
    let mut args = vec!["delete".to_string()];
    args.extend(to_vec_string!(key));
    let shell_result = defaults(to_slice_str!(args))?;
    match shell_result {
        (0 | 1, _, _) => Ok(()),
        (exit_code, _, err) => Err(Error::IOError(format!(
            "defaults {} failed[{}]: {}",
            args.join(" "),
            exit_code,
            err
        ))),
    }
}
pub fn delete_domains(domains: &[&str]) -> Result<DeleteDefaultsMacOSResult> {
    let mut errors = BTreeMap::<String, Error>::new();
    let mut domain_map = export_domains(domains, true)?;

    for domain in domains {
        match defaults_delete_domain(&domain) {
            Ok((domain, plist)) => {
                let path = iocore::Path::raw(&domain).try_canonicalize();
                domain_map.insert(domain.to_string(), (plist, if path.is_file() { Some(path) } else { None }));
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
    pub domain_map: BTreeMap<String, (plist::Value, Option<iocore::Path>)>,
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
pub fn export_domains(
    domains: &[&str],
    global: bool,
) -> Result<BTreeMap<String, (plist::Value, Option<iocore::Path>)>> {
    let mut data = BTreeMap::<String, (plist::Value, Option<iocore::Path>)>::new();
    if global {
        data.insert("NSGlobalDomain".to_string(), (export_domain("NSGlobalDomain")?, None));
    }
    for domain in domains {
        let path = iocore::Path::raw(domain).try_canonicalize();
        data.insert(
            domain.to_string(),
            (export_domain(&domain)?, if path.is_file() { Some(path) } else { None }),
        );
    }
    Ok(data)
}
pub fn export_plists_from_path(
    path: &str,
) -> Result<BTreeMap<String, (plist::Value, Option<iocore::Path>)>> {
    let path_domains =
        iocore::walk_dir(iocore::Path::raw(path), iocore::NoopProgressHandler, None)?
            .iter()
            .filter(|path| {
                path.is_file() && path.extension().unwrap_or_default().ends_with("plist")
            })
            .map(|path| path.to_string())
            .collect::<Vec<String>>();

    Ok(export_domains(to_slice_str!(path_domains), false)?)
}
pub fn export_library_preferences() -> Result<BTreeMap<String, (plist::Value, Option<iocore::Path>)>>
{
    Ok(export_plists_from_path("/Library/Preferences")?)
}
pub fn export_all_domains() -> Result<BTreeMap<String, (plist::Value, Option<iocore::Path>)>> {
    Ok(export_domains(to_slice_str!(list_domains()?), true)?)
}
pub fn defaults(args: &[&str]) -> Result<(i64, String, String)> {
    let (exit_code, stdout, stderr) = defaults_ok(args, None)?;
    match exit_code {
        0 | 1 => Ok((exit_code, stdout, stderr)),
        _ => {
            let command = format!("defaults {}", args.join(" "));
            Err(Error::IOError(format!(
                "command `{}' failed with exit code {}",
                command, exit_code
            )))
        },
    }
}
pub fn defaults_ok(args: &[&str], stdin: Option<Vec<u8>>) -> Result<(i64, String, String)> {
    let mut cmd = Command::new("defaults");
    let cmd = cmd.current_dir("/System");
    let cmd = cmd.args(args);
    let cmd = cmd.stdin(match stdin {
        Some(bytes) => {
            let (read_bytes, mut write_bytes) = pipe()?;
            write_bytes.write_all(&bytes)?;
            Stdio::from(read_bytes)
        },
        None => Stdio::null(),
    });
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

pub fn coredata_fix(quiet: bool, dry_run: bool) -> Result<()> {
    use iocore::Path;
    let settings = CDBSettings::cli(quiet);
    let user_preferences = Path::raw("~/Library/Preferences").try_canonicalize();

    defaults_delete(&["NSGlobalDomain", "NSLinguisticDataAssetsRequested"])?;
    defaults_delete(&["NSGlobalDomain", "NSPreferredWebServices"])?;
    defaults_delete(&["NSGlobalDomain", "AppleInterfaceStyle"])?;
    defaults_delete_domain(user_preferences.join("com.apple.HIToolbox.plist"))?;
    defaults_import_json(
        user_preferences.join("com.apple.HIToolbox.plist"),
        serde_json::json!({
            "AppleEnabledInputSources": [
                {
                    "InputSourceKind": "Keyboard Layout",
                    "KeyboardLayout Name": "USInternational-PC",
                    "KeyboardLayout ID": 15000
                }
            ],
            "AppleSelectedInputSources": [
                {
                    "InputSourceKind": "Keyboard Layout",
                    "KeyboardLayout Name": "USInternational-PC",
                    "KeyboardLayout ID": 15000
                }
            ],
            "AppleInputSourceHistory": [
                {
                    "InputSourceKind": "Keyboard Layout",
                    "KeyboardLayout Name": "USInternational-PC",
                    "KeyboardLayout ID": 15000
                },
                {
                    "InputSourceKind": "Keyboard Layout",
                    "KeyboardLayout Name": "U.S.",
                    "KeyboardLayout ID": 0
                }
            ],
            "AppleCurrentKeyboardLayoutInputSourceID": "com.apple.keylayout.USInternational-PC"
        }),
    )?;

    for args in settings.defaults_exec_args() {
        if dry_run {
            println!("defaults {}", args.join(""));
        } else {
            defaults_ok(to_slice_str!(args), None)?;
            if !quiet {
                eprintln!("defaults {} -", args.join(" "));
            }
        }
    }
    for args in defaults_exec_args() {
        if dry_run {
            println!("defaults {}", args.join(""));
        } else {
            defaults_ok(&args, None)?;
            if !quiet {
                eprintln!("defaults {} -", args.join(" "));
            }
        }
    }

    Ok(())
}
fn defaults_exec_args<'a>() -> Vec<Vec<&'a str>> {
    use iocore::Path;
    let screencapture = Path::raw("~").try_canonicalize().to_string();
    let user_preferences = Path::raw("~/Library/Preferences").try_canonicalize();
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
        vec!["write", "NSGlobaldomain", "AppleKeyboardUIMode", "-integer", "2"],
        vec!["write", "NSGlobaldomain", "AppleLanguages", "-array", "en-US"],
        vec!["write", "NSGlobaldomain", "AppleLocale", "-string", "en-US"],
        vec!["write", "com.apple.dock", "wvous-br-corner", "-bool", "NO"],
        vec!["write", "com.apple.dock", "showAppExposeGestureEnabled", "-bool", "NO"],
        vec!["write", "com.apple.dock", "show-recents", "-bool", "NO"],
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
            user_preferences.join("com.apple.helpd.plist").to_string().leak(),
            "PublicSpotlightIndex",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "AppleMiniaturizeOnDoubleClick",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "com.apple.trackpad.forceClick",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "AppleShowScrollBars",
            "-string",
            "Always",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "AppleShowScrollBars",
            "-string",
            "Always",
        ],
        vec![
            "delete",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "AppleLanguages",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "AppleLanguages",
            "-array",
            "en-US",
        ],
        vec![
            "write",
            user_preferences.join(".GlobalPreferences.plist").to_string().leak(),
            "ContextMenuGesture",
            "-integer",
            "0",
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.systempreferences.plist").to_string().leak(),
            "recentPanes",
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.speech.recognition.AppleSpeechRecognition.CustomCommands.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.businessservicesd.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.remindd.babysitter.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.voicetrigger.notbackedup.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.systempreferences.plist").to_string().leak(),
            "DSKDesktopPrefPane",
            "-dict",
            "UserFolderPaths",
        ],
        vec![
            "write",
            user_preferences.join("com.apple.systempreferences.plist").to_string().leak(),
            "DSKDesktopPrefPane",
            "-dict",
            "UserFolderPaths",
            "-array",
            "/Users/gabrielfalcao",
        ],
        vec!["write", "com.apple.Terminal", "StringEncodings", "-array", "4"],
        vec!["write", "com.apple.screensaver", "askForPassword", "-integer", "1"],
        vec!["write", "com.apple.screensaver", "askForPasswordDelay", "-integer", "0"],
        vec!["write", "NSGlobalDomain", "AppleLanguages", "-array", "\"en-US\""],
        vec!["write", "NSGlobalDomain", "KeyRepeat", "-integer", "1"],
        vec!["write", "NSGlobalDomain", "AppleKeyboardUIMode", "-integer", "2"],
        vec!["write", "NSGlobalDomain", "InitialKeyRepeat", "-integer", "1"],
        vec!["write", "NSGlobalDomain", "com.apple.keyboard.fnState", "-integer", "0"],
        vec!["delete", "NSGlobalDomain", "NSLinguisticDataAssetsRequestedByChecker"],
        vec![
            "write",
            "NSGlobalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
            "-array",
            "us",
        ],
        vec![
            "write",
            "NSGlobalDomain",
            "NSWebServicesProviderWebSearch",
            "-dict",
            "NSWebServicesProviderWebSearch",
        ],
        vec![
            "write",
            "NSGlobalDomain",
            "NSSpellCheckerContainerTransitionComplete",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "NSGlobalDomain",
            "NSSpellCheckerDictionaryContainerTransitionComplete",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            "NSGlobalDomain",
            "NSUserQuotesArray",
            "-array",
            "\\U201c",
            "\\U201d",
            "\\U2018",
            "\\U2019",
        ],
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
        vec![
            "write",
            user_preferences
                .join("com.apple.MobileBluetooth.debug.plist")
                .to_string()
                .leak(),
            "LeDeviceCache",
            "-dict",
            "WipeNameOrigin",
            "bool",
            "YES",
        ],
        vec![
            "write",
            user_preferences
                .join("com.apple.Sharing-Settings.extension.plist")
                .to_string()
                .leak(),
            "com.apple.preferences.sharing.allowFullDiskAccess",
            "bool",
            "NO",
        ],
        vec![
            "write",
            user_preferences
                .join("com.apple.diagnosticextensionsd.plist")
                .to_string()
                .leak(),
            "directoriesCleanupDone",
            "bool",
            "YES",
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.inputAnalytics.IASGenmojiAnalyzer.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "write",
            user_preferences.join("com.apple.BTServer.plist").to_string().leak(),
            "defaultPoweredState",
            "-bool",
            "NO",
        ],
        vec![
            "write",
            user_preferences.join("com.apple.BTServer.plist").to_string().leak(),
            "defaultAirplaneModePowerState",
            "-bool",
            "NO",
        ],
        vec![
            "delete",
            user_preferences.join("com.googlecode.iterm2.private.plist").to_string().leak(),
        ],
        vec!["delete", user_preferences.join("com.apple.shazamd.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences
                .join("com.apple.inputAnalytics.IASSRAnalyzer.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.inputAnalytics.IASWTAnalyzer.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.assistant.support.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.EmojiCache.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.facetime.bag.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.AMPLibraryAgent.plist").to_string().leak(),
        ],
        vec!["delete", user_preferences.join("com.apple.assistant.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.sharingd.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.Siri.plist").to_string().leak()],
        vec![
            "write",
            user_preferences.join("com.apple.stockholm.plist").to_string().leak(),
            "RemoteAdminV2",
            "-bool",
            "NO",
        ],
        vec!["delete", user_preferences.join("com.apple.stockholm.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.remindd.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences.join("com.apple.EmojiPreferences.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.PhotoBooth.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.homed.notbackedup.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.NewDeviceOutreach.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.NewDeviceOutreach.plist").to_string().leak(),
        ],
        vec!["delete", user_preferences.join("ContextStoreAgent.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.iTunes.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences
                .join("com.apple.Safari.SandboxBroker.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.VideoSubscriberAccount.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.messages.nicknames.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.donotdisturbd.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.AccessibilityHearingNearby.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.weather.sensitive.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.assistant.backedup.plist").to_string().leak(),
        ],
        vec!["delete", user_preferences.join("com.apple.iChat.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences.join("com.firebase.FIRInstallations.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.siri.sirisuggestions.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.DuetExpertCenter.AppPredictionExpert.plist")
                .to_string()
                .leak(),
        ],
        vec!["delete", user_preferences.join("com.apple.mmcs.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences.join("com.apple.voicememod.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.preferences.extensions.ShareMenu.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "write",
            user_preferences
                .join("com.apple.preferences.extensions.ShareMenu.plist")
                .to_string()
                .leak(),
            "displayOrder",
            "com.apple.share.AirDrop.send",
        ],
        vec!["delete", user_preferences.join("com.apple.newscore2.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.iPod.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences
                .join("com.apple.GenerativeFunctions.GenerativeFunctionsInstrumentation.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.knowledge-agent.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.xpc.activity2.plist").to_string().leak(),
            "ActivityBaseDates",
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.xpc.activity2.plist").to_string().leak(),
            "VersionSpecificActivitiesRun",
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.IMCoreSpotlight.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.systemuiserver.plist").to_string().leak(),
            "last-analytics-stamp",
        ],
        vec!["delete", user_preferences.join("com.apple.studentd.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.Wallet.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences.join("org.videolan.vlc.plist").to_string().leak(),
            "recentlyPlayedMedia",
        ],
        vec![
            "delete",
            user_preferences.join("org.videolan.vlc.plist").to_string().leak(),
            "recentlyPlayedMediaList",
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.HearingAids.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.apple.findmy.findmylocateagent.plist")
                .to_string()
                .leak(),
        ],
        vec!["delete", user_preferences.join("mbuseragent.plist").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.parsecd").to_string().leak()],
        vec!["delete", user_preferences.join("com.apple.homed.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences.join("com.apple.videosubscriptionsd.plist").to_string().leak(),
        ],
        vec![
            "delete",
            user_preferences.join("com.apple.identityservicesd.plist").to_string().leak(),
        ],
        vec!["delete", user_preferences.join("com.apple.Maps.plist").to_string().leak()],
        vec![
            "delete",
            user_preferences
                .join("com.apple.CloudSubscriptionFeatures.diagnostic.plist")
                .to_string()
                .leak(),
        ],
        vec![
            "delete",
            user_preferences
                .join("com.ikmultimedia.Product Manager.plist")
                .to_string()
                .leak(),
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
        let domains: BTreeMap<String, (plist::Value, Option<iocore::Path>)> = export_all_domains()?;
        assert_eq!(domains.is_empty(), false);
        assert_eq!(domains.contains_key(&"com.apple.Safari".to_string()), true);
        let safari = match domains.get("com.apple.Safari").unwrap() {
            (Value::Dictionary(safari), _) => safari.clone(),
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

pub fn validate_domain_path_for_current_user(path: impl std::fmt::Display) -> Result<()> {
    use iocore::Path;
    let path = Path::raw(path.to_string()).try_canonicalize();
    let (_, parts) = path.search_regex("^/Users/(?<user>[^/]+)/")?;
    let user_home_username = parts[0].to_string();
    let current_user = iocore::User::id().unwrap_or_default();

    if user_home_username.len() > 0 {
        if user_home_username.to_string() == current_user.name.to_string() {
            Ok(())
        } else {
            return Err(Error::CoreDataError(format!(
                "domain file {} does not belong to user {:#?}",
                path.to_string(),
                &current_user.name
            )));
        }
    } else {
        Ok(()) // Path not in user home
    }
}
pub fn defaults_import_json(domain: impl std::fmt::Display, json: serde_json::Value) -> Result<()> {
    use std::io::BufWriter;
    validate_domain_path_for_current_user(&domain)?;
    let args = vec!["import".to_string(), domain.to_string(), "-".to_string()];
    let mut writer = BufWriter::new(Vec::<u8>::new());
    plist::to_writer_xml(&mut writer, &json)?;
    let (exit_code, _, err) = defaults_ok(to_slice_str!(args), Some(writer.into_inner()?))?;
    if exit_code != 0 {
        return Err(Error::IOError(format!(
            "defaults export {} failed[{}]: {}",
            &domain, exit_code, err
        )));
    }
    Ok(())
}

#[macro_export]
macro_rules! to_vec_string {
    ($slice:expr) => {
        $slice.iter().map(|j| j.to_string()).collect::<Vec<String>>()
    };
}
#[macro_export]
macro_rules! to_slice_str {
    ($vec_string:expr) => {
        &$vec_string.iter().map(|j| j.as_str()).collect::<Vec<&str>>()
    };
}
