use std::env::consts;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use regex::Regex;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use self_update::backends::github::{ReleaseList, UpdateBuilder};
use thiserror::Error;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[derive(Debug, Clone, Copy, Error)]
pub enum UpdateError {
    #[error("Not supported on the platform: {0}")]
    NotSupported(&'static str),
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Version tag not in the format of '(v)X.Y.Z'")]
    VersionTag,
    #[error("No release is available for the target")]
    MissingRelease,
    #[error("{0}")]
    SelfUpdate(#[from] self_update::errors::Error),
}

#[allow(unused_variables)]
#[allow(clippy::missing_panics_doc)]
pub fn update(
    bin_name: &str,
    current_version: &str,
    target_version: Option<&str>,
) -> Result<(), UpdateError> {
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(UpdateError::NotSupported(consts::OS))
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let mut version = if let Some(version) = target_version {
            let version = version.to_owned();
            let re = Regex::new(r"^v?[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
            if !re.is_match(&version) {
                return Err(UpdateError::VersionTag);
            }
            version
        } else {
            let releases = ReleaseList::configure()
                .repo_owner("amtep")
                .repo_name("ck3-tiger")
                .with_target(consts::OS)
                .build()?
                .fetch()?;

            releases.first().ok_or(UpdateError::MissingRelease)?.version.clone()
        };

        if !version.starts_with('v') {
            version.insert(0, 'v');
        }

        #[cfg(target_os = "linux")]
        let bin_path = format!("{0}-{1}-{2}/{0}", bin_name, consts::OS, version);
        #[cfg(target_os = "windows")]
        let bin_path = format!("{}.exe", bin_name);

        let mut updater = UpdateBuilder::new();
        updater
            .repo_owner("amtep")
            .repo_name("ck3-tiger")
            .bin_name(bin_name)
            .bin_path_in_archive(&bin_path)
            .identifier(bin_name)
            .target(consts::OS)
            .current_version(current_version)
            .show_download_progress(true);

        if target_version.is_some() {
            updater.target_version_tag(&version);
        }
        updater.build()?.update()?;

        Ok(())
    }
}
