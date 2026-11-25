//! Which Keeplight this is, and the scratch guard.
//!
//! A release build from a full checkout (the installer's) is the release app:
//! `%LOCALAPPDATA%\Keeplight`, the startup entry `Keeplight`, OpenRGB on
//! 127.0.0.1:6742 and the update check. Any other build, debug or from a git
//! worktree (`build.rs`), is the dev app: `%LOCALAPPDATA%\Keeplight Dev`, the
//! startup entry `Keeplight Dev` (so its keeper's mutex and stop event are its
//! own), and no real OpenRGB, update check or download unless
//! `KEEPLIGHT_NETWORK=live` is set by name.
//!
//! `KEEPLIGHT_SCRATCH=1` (every test and smoke test) refuses, rather than
//! warns: a data folder that isn't named, is relative, or is under %APPDATA%
//! or %LOCALAPPDATA% outside the temp folder; an OpenRGB that isn't named,
//! isn't on loopback or is the real port 6742; a startup entry that isn't a
//! test name; and `KEEPLIGHT_NETWORK=live`.

use std::net::ToSocketAddrs;
use std::path::{Component, Path, PathBuf};

use crate::sdk::DEFAULT_ADDRESS;

/// The release app's folder and startup entry name.
pub const RELEASE_NAME: &str = "Keeplight";
/// The dev app's folder and startup entry name.
pub const DEV_NAME: &str = "Keeplight Dev";
/// OpenRGB's real SDK port, which only the release app uses by default.
pub const REAL_PORT: u16 = 6742;

/// Whether this binary is the release app (`build.rs`).
pub fn is_release_build() -> bool {
    env!("KEEPLIGHT_IDENTITY") == "release"
}

/// Why the guard refused to start, in words for the person running it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Refused(pub String);

/// Everything the guard reads, so tests can build one by hand.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Env {
    pub release: bool,
    pub scratch: bool,
    /// `KEEPLIGHT_DATA_DIR`, or its older name `KEEPLIGHT_DATA`.
    pub data_dir: Option<PathBuf>,
    pub openrgb: Option<String>,
    pub network: Option<String>,
    pub secrets: Option<String>,
    pub startup_name: Option<String>,
    pub local_app_data: Option<PathBuf>,
    pub app_data: Option<PathBuf>,
    pub temp: PathBuf,
}

impl Env {
    pub fn from_env() -> Env {
        let text = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let path = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        Env {
            release: is_release_build(),
            scratch: text("KEEPLIGHT_SCRATCH").as_deref() == Some("1"),
            data_dir: path("KEEPLIGHT_DATA_DIR").or_else(|| path("KEEPLIGHT_DATA")),
            openrgb: text("KEEPLIGHT_OPENRGB"),
            network: text("KEEPLIGHT_NETWORK"),
            secrets: text("KEEPLIGHT_SECRETS"),
            startup_name: text("KEEPLIGHT_STARTUP_NAME"),
            local_app_data: path("LOCALAPPDATA"),
            app_data: path("APPDATA"),
            temp: std::env::temp_dir(),
        }
    }

    fn name(&self) -> &'static str {
        if self.release { RELEASE_NAME } else { DEV_NAME }
    }

    /// The data folder: the one named, else `%LOCALAPPDATA%\Keeplight` (or
    /// `Keeplight Dev`), else that name in the temp folder.
    pub fn data(&self) -> PathBuf {
        self.data_dir
            .clone()
            .unwrap_or_else(|| self.local_app_data.clone().unwrap_or_else(|| self.temp.clone()).join(self.name()))
    }

    /// OpenRGB's SDK address: the one named, else 127.0.0.1:6742.
    pub fn openrgb(&self) -> String {
        self.openrgb.clone().unwrap_or_else(|| DEFAULT_ADDRESS.to_string())
    }

    /// The startup entry's name, which the keeper's mutex and stop event
    /// follow: the one named, else `Keeplight` (or `Keeplight Dev`).
    pub fn startup_name(&self) -> String {
        self.startup_name.clone().unwrap_or_else(|| self.name().to_string())
    }

    /// Whether calls off this PC (the update check, downloads) may run:
    /// `KEEPLIGHT_NETWORK=live`, or unset in a release build. Never in scratch.
    pub fn network_live(&self) -> bool {
        !self.scratch
            && match self.network.as_deref() {
                Some(v) => v == "live",
                None => self.release,
            }
    }

    /// Whether Keeplight may talk to OpenRGB at `address`. The real port, or
    /// another computer, is refused in scratch, and in a dev build unless
    /// `KEEPLIGHT_NETWORK=live`.
    pub fn may_connect(&self, address: &str) -> Result<(), Refused> {
        if !is_real_openrgb(address) {
            return Ok(());
        }
        if self.scratch {
            return Err(Refused(format!(
                "Scratch mode doesn't talk to the real OpenRGB ({address}). Set KEEPLIGHT_OPENRGB to a loopback \
                 port the test owns, other than {REAL_PORT}."
            )));
        }
        if !self.release && self.network.as_deref() != Some("live") {
            return Err(Refused(format!(
                "This dev build of Keeplight doesn't talk to the real OpenRGB ({address}) unless KEEPLIGHT_NETWORK=live \
                 is set. Point KEEPLIGHT_OPENRGB at a test server instead."
            )));
        }
        Ok(())
    }

    /// Whether the startup entry may be written under its name.
    pub fn may_write_startup(&self) -> Result<(), Refused> {
        let name = self.startup_name();
        let real = |n: &str| name.eq_ignore_ascii_case(n);
        if self.scratch && (self.startup_name.is_none() || real(RELEASE_NAME) || real(DEV_NAME)) {
            return Err(Refused(
                "Scratch mode doesn't touch a real startup entry. Set KEEPLIGHT_STARTUP_NAME to a test name, such as \
                 Keeplight-smoke-1."
                    .into(),
            ));
        }
        if !self.release && real(RELEASE_NAME) {
            return Err(Refused(
                "A dev build of Keeplight never uses the installed Keeplight's startup entry or keeper.".into(),
            ));
        }
        Ok(())
    }

    /// Every check, before a front end starts.
    pub fn check(&self) -> Result<(), Refused> {
        if let Some(n) = self.network.as_deref().filter(|n| !matches!(*n, "live" | "offline")) {
            return Err(Refused(format!("KEEPLIGHT_NETWORK is \u{201c}{n}\u{201d}; it can be live or offline.")));
        }
        if let Some(s) = self.secrets.as_deref().filter(|s| !matches!(*s, "keychain" | "memory")) {
            return Err(Refused(format!("KEEPLIGHT_SECRETS is \u{201c}{s}\u{201d}; it can be keychain or memory.")));
        }
        let data = self.data();
        if !self.release
            && let Some(local) = &self.local_app_data
            && is_within(&data, &local.join(RELEASE_NAME))
        {
            return Err(Refused(format!(
                "A dev build of Keeplight never opens the installed Keeplight's folder ({}).",
                data.display()
            )));
        }
        if self.scratch {
            if self.network.as_deref() == Some("live") {
                return Err(Refused("Scratch mode is offline: KEEPLIGHT_NETWORK=live is refused.".into()));
            }
            if self.secrets.as_deref() == Some("keychain") {
                return Err(Refused(
                    "Scratch mode keeps secrets in memory: KEEPLIGHT_SECRETS=keychain is refused.".into(),
                ));
            }
            if self.data_dir.is_none() {
                return Err(Refused("Scratch mode needs KEEPLIGHT_DATA_DIR, a throwaway folder.".into()));
            }
            if !data.is_absolute() || data.components().any(|c| c == Component::ParentDir) {
                return Err(Refused(format!(
                    "Scratch mode needs KEEPLIGHT_DATA_DIR as a full path without \u{201c}..\u{201d}, not {}.",
                    data.display()
                )));
            }
            let profile = [&self.local_app_data, &self.app_data];
            if !is_within(&data, &self.temp) && profile.iter().any(|p| p.as_ref().is_some_and(|p| is_within(&data, p)))
            {
                return Err(Refused(format!(
                    "Scratch mode doesn't open {}: it is in the real app data folders. Use a folder under {}.",
                    data.display(),
                    self.temp.display()
                )));
            }
            if self.openrgb.is_none() {
                return Err(Refused("Scratch mode needs KEEPLIGHT_OPENRGB, a fake OpenRGB the test owns.".into()));
            }
        }
        self.may_write_startup()?;
        self.may_connect(&self.openrgb())
    }
}

/// The real OpenRGB: port 6742, another computer, or an address that can't be
/// read (refused rather than guessed).
pub fn is_real_openrgb(address: &str) -> bool {
    match address.to_socket_addrs() {
        Ok(addrs) => {
            let addrs: Vec<_> = addrs.collect();
            addrs.is_empty() || addrs.iter().any(|a| !a.ip().is_loopback() || a.port() == REAL_PORT)
        }
        Err(_) => true,
    }
}

/// Whether `path` is `base` or inside it, after following what exists of
/// each (so a short 8.3 name or a link can't hide one), ignoring case on
/// Windows.
pub fn is_within(path: &Path, base: &Path) -> bool {
    fold(&resolve(path)).starts_with(fold(&resolve(base)))
}

fn resolve(path: &Path) -> PathBuf {
    let mut rest = Vec::new();
    let mut at = path.to_path_buf();
    loop {
        if let Ok(real) = std::fs::canonicalize(&at) {
            return rest.iter().rev().fold(real, |p, name| p.join(name));
        }
        match (at.parent(), at.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_os_string());
                at = parent.to_path_buf();
            }
            _ => return path.to_path_buf(),
        }
    }
}

fn fold(path: &Path) -> PathBuf {
    if cfg!(windows) { PathBuf::from(path.to_string_lossy().to_lowercase()) } else { path.to_path_buf() }
}

/// Every front end's first step: the checks, then the data folder, made if
/// needed and limited to the current user.
pub fn launch() -> Result<Env, Refused> {
    let env = Env::from_env();
    env.check()?;
    let data = env.data();
    prepare_data_dir(&data)
        .map_err(|e| Refused(format!("Keeplight couldn't set up its folder {}: {e}", data.display())))?;
    Ok(env)
}

/// Creates the data folder if needed and limits it to the current user, on a
/// new folder and on an existing one alike.
pub fn prepare_data_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    restrict_to_user(dir)
}

/// Replaces the folder's access list with one entry, full control for the
/// current user, inherited by everything in it. Elevated runs (the OpenRGB
/// installer) are the same user, so nothing else needs access.
#[cfg(windows)]
pub fn restrict_to_user(dir: &Path) -> std::io::Result<()> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
    };
    use windows::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl, PROTECTED_DACL_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR,
    };
    use windows::core::{BOOL, HSTRING, PCWSTR};

    let sddl = HSTRING::from(format!("D:P(A;OICI;FA;;;{})", windows_acl::current_user_sid()?));
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: a valid SDDL string; the descriptor is freed below.
    unsafe { ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None) }
        .map_err(std::io::Error::other)?;
    let (mut present, mut defaulted) = (BOOL(0), BOOL(0));
    let mut dacl: *mut ACL = std::ptr::null_mut();
    // SAFETY: `sd` is the descriptor made above; `dacl` points into it.
    let result = unsafe { GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted) }
        .map_err(std::io::Error::other)
        .and_then(|()| {
            let path = HSTRING::from(dir.as_os_str());
            // SAFETY: a NUL-terminated path and the DACL from `sd`, alive for the call.
            unsafe {
                SetNamedSecurityInfoW(
                    PCWSTR(path.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(dacl),
                    None,
                )
            }
            .ok()
            .map_err(std::io::Error::other)
        });
    // SAFETY: frees the descriptor LocalAlloc'd by the conversion.
    unsafe { LocalFree(Some(HLOCAL(sd.0))) };
    result
}

#[cfg(not(windows))]
pub fn restrict_to_user(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

#[cfg(windows)]
pub mod windows_acl {
    use windows::Win32::Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::core::PWSTR;

    /// The current user's SID, such as `S-1-5-21-…-1001`.
    pub fn current_user_sid() -> std::io::Result<String> {
        let mut token = HANDLE::default();
        // SAFETY: the process's own token, closed below.
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.map_err(std::io::Error::other)?;
        let mut size = 0u32;
        // SAFETY: a size query; it fails with the size needed.
        let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut size) };
        // u64s, so the TOKEN_USER read below is aligned.
        let mut buf = vec![0u64; (size as usize).div_ceil(8)];
        // SAFETY: `buf` holds at least `size` bytes.
        let read = unsafe { GetTokenInformation(token, TokenUser, Some(buf.as_mut_ptr().cast()), size, &mut size) };
        // SAFETY: the handle was opened above.
        let _ = unsafe { CloseHandle(token) };
        read.map_err(std::io::Error::other)?;
        // SAFETY: GetTokenInformation filled `buf` with a TOKEN_USER.
        let user = unsafe { &*buf.as_ptr().cast::<TOKEN_USER>() };
        let mut text = PWSTR::null();
        // SAFETY: the SID points into `buf`; the string is freed below.
        unsafe { ConvertSidToStringSidW(user.User.Sid, &mut text) }.map_err(std::io::Error::other)?;
        // SAFETY: a NUL-terminated string from the conversion.
        let sid = unsafe { text.to_string() }.map_err(std::io::Error::other);
        // SAFETY: LocalAlloc'd by the conversion.
        unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
        sid
    }

    /// The folder's access list as SDDL, such as `D:PAI(A;OICI;FA;;;S-1-…)`.
    pub fn dacl_sddl(dir: &std::path::Path) -> std::io::Result<String> {
        use windows::Win32::Security::Authorization::{
            ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, SDDL_REVISION_1,
            SE_FILE_OBJECT,
        };
        use windows::Win32::Security::{DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};
        use windows::core::{HSTRING, PCWSTR};

        let path = HSTRING::from(dir.as_os_str());
        let mut sd = PSECURITY_DESCRIPTOR::default();
        // SAFETY: a NUL-terminated path; the descriptor is freed below.
        unsafe {
            GetNamedSecurityInfoW(
                PCWSTR(path.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                None,
                None,
                &mut sd,
            )
        }
        .ok()
        .map_err(std::io::Error::other)?;
        let mut text = PWSTR::null();
        // SAFETY: `sd` is the descriptor read above.
        let made = unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                sd,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut text,
                None,
            )
        };
        let sddl = made.map_err(std::io::Error::other).and_then(|()| {
            // SAFETY: a NUL-terminated string from the conversion.
            let s = unsafe { text.to_string() }.map_err(std::io::Error::other);
            // SAFETY: LocalAlloc'd by the conversion.
            unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
            s
        });
        // SAFETY: LocalAlloc'd by GetNamedSecurityInfoW.
        unsafe { LocalFree(Some(HLOCAL(sd.0))) };
        sddl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("keeplight-scratch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// A scratch run as the smoke test sets it up, on a made-up profile
    /// outside the temp folder.
    fn scratch() -> Env {
        let root = std::env::temp_dir().ancestors().last().unwrap().to_path_buf();
        let profile = root.join(format!("keeplight-fake-profile-{}", std::process::id()));
        Env {
            release: false,
            scratch: true,
            data_dir: Some(std::env::temp_dir().join("keeplight-test-data")),
            openrgb: Some("127.0.0.1:50123".into()),
            network: Some("offline".into()),
            secrets: None,
            startup_name: Some("Keeplight-smoke-1".into()),
            local_app_data: Some(profile.join("Local")),
            app_data: Some(profile.join("Roaming")),
            temp: std::env::temp_dir(),
        }
    }

    fn refused(env: &Env) -> String {
        env.check().expect_err("refused").0
    }

    #[test]
    fn tests_run_in_scratch() {
        // `.cargo/config.toml` sets it for every cargo test, so a test that
        // reaches for the real OpenRGB or folder is refused.
        assert!(Env::from_env().scratch, "KEEPLIGHT_SCRATCH=1 should be set for tests");
        assert!(!Env::from_env().network_live());
    }

    #[test]
    fn a_full_scratch_setup_passes() {
        assert_eq!(scratch().check(), Ok(()));
        assert!(!scratch().network_live());
    }

    #[test]
    fn scratch_refuses_the_real_openrgb() {
        for address in ["127.0.0.1:6742", "localhost:6742", "[::1]:6742", "192.168.1.20:50123", "not an address"] {
            let env = Env { openrgb: Some(address.into()), ..scratch() };
            assert!(refused(&env).contains("real OpenRGB"), "{address}");
        }
        let env = Env { openrgb: None, ..scratch() };
        assert!(refused(&env).contains("KEEPLIGHT_OPENRGB"));
        // Live doesn't lift it in scratch.
        let env = Env { network: Some("live".into()), ..scratch() };
        assert!(env.may_connect("127.0.0.1:6742").is_err());
        assert!(refused(&env).contains("offline"));
    }

    #[test]
    fn scratch_refuses_real_folders() {
        let s = scratch();
        let local = s.local_app_data.clone().unwrap();
        let roaming = s.app_data.clone().unwrap();
        for dir in [local.join("Keeplight"), local.join("Keeplight Dev"), local.clone(), roaming.join("x")] {
            let env = Env { data_dir: Some(dir.clone()), ..scratch() };
            assert!(env.check().is_err(), "{}", dir.display());
        }
        let env = Env { data_dir: None, ..scratch() };
        assert!(refused(&env).contains("KEEPLIGHT_DATA_DIR"));
        let env = Env { data_dir: Some(PathBuf::from("relative")), ..scratch() };
        assert!(refused(&env).contains("full path"));
        let sneaky = std::env::temp_dir().join("..").join("Keeplight");
        let env = Env { data_dir: Some(sneaky), ..scratch() };
        assert!(refused(&env).contains("full path"));
    }

    #[test]
    fn temp_inside_the_profile_is_still_allowed() {
        // Windows' temp folder is %LOCALAPPDATA%\Temp.
        let s = scratch();
        let local = s.local_app_data.clone().unwrap();
        let env = Env { temp: local.join("Temp"), data_dir: Some(local.join("Temp").join("k")), ..scratch() };
        assert_eq!(env.check(), Ok(()));
    }

    #[test]
    fn scratch_refuses_the_real_startup_entries() {
        for name in [None, Some("Keeplight"), Some("keeplight dev")] {
            let env = Env { startup_name: name.map(str::to_string), ..scratch() };
            assert!(refused(&env).contains("KEEPLIGHT_STARTUP_NAME"), "{name:?}");
        }
    }

    #[test]
    fn a_dev_build_has_its_own_names_and_stays_off_the_real_openrgb() {
        let dev = Env { scratch: false, data_dir: None, openrgb: None, network: None, startup_name: None, ..scratch() };
        assert_eq!(dev.data(), dev.local_app_data.clone().unwrap().join("Keeplight Dev"));
        assert_eq!(dev.startup_name(), "Keeplight Dev");
        assert!(!dev.network_live());
        assert!(refused(&dev).contains("KEEPLIGHT_NETWORK=live"));
        let live = Env { network: Some("live".into()), ..dev.clone() };
        assert_eq!(live.check(), Ok(()));
        assert!(live.network_live());
        let fake = Env { openrgb: Some("127.0.0.1:50123".into()), ..dev.clone() };
        assert_eq!(fake.check(), Ok(()));
        // Never the installed app's folder or keeper, live or not.
        let local = dev.local_app_data.clone().unwrap();
        let env = Env { data_dir: Some(local.join("Keeplight").join("x")), ..live.clone() };
        assert!(refused(&env).contains("installed Keeplight's folder"));
        let env = Env { startup_name: Some("Keeplight".into()), ..live };
        assert!(refused(&env).contains("startup entry"));
    }

    #[test]
    fn the_release_build_keeps_its_names_and_defaults() {
        let release = Env {
            release: true,
            scratch: false,
            data_dir: None,
            openrgb: None,
            network: None,
            startup_name: None,
            ..scratch()
        };
        assert_eq!(release.data(), release.local_app_data.clone().unwrap().join("Keeplight"));
        assert_eq!(release.startup_name(), "Keeplight");
        assert_eq!(release.openrgb(), "127.0.0.1:6742");
        assert!(release.network_live());
        assert_eq!(release.check(), Ok(()));
        let offline = Env { network: Some("offline".into()), ..release.clone() };
        assert!(!offline.network_live());
        // Scratch applies to a release build too, as in CI's smoke test.
        let env = Env { scratch: true, openrgb: Some("127.0.0.1:6742".into()), ..scratch() };
        assert!(Env { release: true, ..env }.check().is_err());
    }

    #[test]
    fn bad_values_are_named() {
        let env = Env { network: Some("on".into()), ..scratch() };
        assert!(refused(&env).contains("KEEPLIGHT_NETWORK"));
        let env = Env { secrets: Some("keychain".into()), ..scratch() };
        assert!(refused(&env).contains("KEEPLIGHT_SECRETS"));
    }

    #[test]
    fn the_data_folder_is_the_users_alone() {
        let d = temp("acl");
        prepare_data_dir(&d).unwrap();
        // Again on an existing folder, as at every launch.
        prepare_data_dir(&d).unwrap();
        #[cfg(windows)]
        {
            let sddl = windows_acl::dacl_sddl(&d).unwrap();
            let sid = windows_acl::current_user_sid().unwrap();
            assert!(sddl.starts_with("D:P"), "inheritance from the parent is cut: {sddl}");
            assert_eq!(sddl.matches('(').count(), 1, "one entry only: {sddl}");
            assert!(sddl.contains(&format!(";;;{sid})")), "the entry is the current user's: {sddl}");
            // A file made inside inherits the same single entry.
            let f = d.join("setup.json");
            std::fs::write(&f, "{}").unwrap();
            let file = windows_acl::dacl_sddl(&f).unwrap();
            assert_eq!(file.matches('(').count(), 1, "{file}");
            assert!(file.contains(&sid), "{file}");
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&d).unwrap().permissions().mode() & 0o777, 0o700);
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}
