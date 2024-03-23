use regex::Regex;
use self_update::backends::github::UpdateBuilder;
#[allow(unused_imports)]
pub use self_update::Status;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Not supported on the platform: {0}")]
    NotSupported(&'static str),
    #[error("Version tag not in the format of 'vX.Y.Z'")]
    VersionTag,
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
        let mut builder = UpdateBuilder::new();
        builder
            .repo_owner("amtep")
            .repo_name("ck3-tiger")
            .bin_name(bin_name)
            .bin_path_in_archive("{{bin}}-{{target}}-v{{version}}/{{bin}}")
            .identifier(bin_name)
            .target(std::env::consts::OS)
            .current_version(current_version)
            .show_download_progress(true)
            .no_confirm(true);

        if let Some(version) = target_version {
            let re = Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
            if !re.is_match(version) {
                return Err(UpdateError::VersionTag);
            }
            builder.target_version_tag(version);
        }

        builder.build()?.update()?;
        Ok(())
    }
}
