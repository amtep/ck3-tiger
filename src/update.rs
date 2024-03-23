use regex::Regex;
use self_update::backends::github::{ReleaseList, UpdateBuilder};
#[allow(unused_imports)]
pub use self_update::Status;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Not supported on the platform: {0}")]
    NotSupported(&'static str),
    #[error("Version tag not in the format of '(v)X.Y.Z'")]
    VersionTag,
    #[error("No release is available for the target")]
    MissingRelease,
    #[error("{0}")]
    SelfUpdate(#[from] self_update::errors::Error),
}

#[allow(clippy::missing_panics_doc)]
pub fn update(
    bin_name: &str,
    current_version: &str,
    target_version: Option<&str>,
) -> Result<(), UpdateError> {
    if cfg!(not(any(target_os = "windows", target_os = "linux"))) {
        Err(UpdateError::NotSupported(std::env::consts::OS))
    } else {
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
                .with_target(std::env::consts::OS)
                .build()?
                .fetch()?;

            releases.first().ok_or(UpdateError::MissingRelease)?.version.clone()
        };

        if !version.starts_with('v') {
            version.insert(0, 'v');
        }

        let bin_path = format!("{0}-{1}-{2}/{0}", bin_name, std::env::consts::OS, version);
        UpdateBuilder::new()
            .repo_owner("amtep")
            .repo_name("ck3-tiger")
            .bin_name(bin_name)
            .bin_path_in_archive(&bin_path)
            .identifier(bin_name)
            .target(std::env::consts::OS)
            .target_version_tag(&version)
            .current_version(current_version)
            .show_download_progress(true)
            .no_confirm(true)
            .build()?
            .update()?;
        Ok(())
    }
}
