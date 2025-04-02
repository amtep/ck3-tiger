use std::env::consts;

use cfg_if::cfg_if;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use regex::Regex;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use self_update::backends::github::UpdateBuilder;
use thiserror::Error;

cfg_if! {
    if #[cfg(any(target_os = "windows", target_os = "linux"))] {
        #[derive(Debug, Error)]
        pub enum UpdateError {
            #[error("Version tag not in the format of '(v)X.Y.Z'")]
            VersionTag,
            #[error("{0}")]
            SelfUpdate(#[from] self_update::errors::Error),
        }
    }
    else {
        #[derive(Debug, Error)]
        pub enum UpdateError {
            #[error("Not supported on the platform: {0}")]
            NotSupported(&'static str),
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
cfg_if! {
    if #[cfg(feature = "ck3")] {
        const BIN_NAME: &str = "ck3-tiger";
    } else if #[cfg(feature = "vic3")] {
        const BIN_NAME: &str = "vic3-tiger";
    } else if #[cfg(feature = "imperator")] {
        const BIN_NAME: &str = "imperator-tiger";
    } else if #[cfg(feature = "hoi4")] {
        const BIN_NAME: &str = "hoi4-tiger";
    }
}

/// Self-update the main tiger application.
///
/// `current_version` is the current version of the application, and may be obtained by using `env!("CARGO_PKG_VERSION")`
/// from within the cargo package containing the binary crate.
///
/// If `target_version` is `Some(ver)`, then it will force update to the specified version. Otherwise, the latest release will
/// be fetched and installed **only** if the latest release version is greater than the current version.
#[allow(dead_code)]
pub fn update(current_version: &str, target_version: Option<&str>) -> Result<(), UpdateError> {
    cfg_if! {
        if #[cfg(any(target_os = "windows", target_os = "linux"))] {
            if let Some(version) = target_version {
                let re = Regex::new(r"^v?[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
                if !re.is_match(version) {
                    return Err(UpdateError::VersionTag);
                }
            }

            #[cfg(target_os = "linux")]
            let bin_path = format!("{BIN_NAME}-linux-v{{{{version}}}}/{BIN_NAME}");
            #[cfg(target_os = "windows")]
            let bin_path = format!("{}.exe", BIN_NAME);

            let mut updater = UpdateBuilder::new();
            updater
                .repo_owner("amtep")
                .repo_name("ck3-tiger")
                .bin_name(BIN_NAME)
                .bin_path_in_archive(&bin_path)
                .identifier(BIN_NAME)
                .target(consts::OS)
                .current_version(current_version)
                .show_download_progress(true);

            if let Some(version) = target_version {
                let mut version = version.to_owned();
                if !version.starts_with('v') {
                    version.insert(0, 'v');
                }
                updater.target_version_tag(&version);
            }
            updater.build()?.update()?;

            Ok(())
        } else {
            Err(UpdateError::NotSupported(consts::OS))
        }
    }
}
