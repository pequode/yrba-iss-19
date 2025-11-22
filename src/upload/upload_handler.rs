use super::sftp::upload_sftp;
use crate::config::Config;
use crate::upload::file_copy::file_copy_backup;
use crate::upload::nfs_copy::nfs_copy_backup;
use anyhow::anyhow;
use std::path::Path;
use url::Url;

#[derive(Clone)]
pub(crate) enum UploadMode {
    File,
    Sftp,
    Nfs,
}

pub(crate) fn get_upload_mode(remote_str: &str) -> anyhow::Result<UploadMode> {
    let url: Url = Url::parse(remote_str).expect("Could not parse remote URL!");
    let upload_mode = match url.scheme() {
        "file" => UploadMode::File,
        "sftp" => UploadMode::Sftp,
        "nfs" => UploadMode::Nfs,
        "smb" => todo!("No SMB support yet!"),
        _ => return Err(anyhow!("Unknown upload mode: {}", url.scheme())),
    };
    Ok(upload_mode)
}

pub(crate) fn upload_file(
    file_path: &Path,
    upload_mode: &UploadMode,
    config: &Config,
) -> anyhow::Result<()> {
    log::info!("Starting upload...");
    match upload_mode {
        UploadMode::Sftp => upload_sftp(file_path, config)?,
        UploadMode::File => file_copy_backup(file_path, config)?,
        UploadMode::Nfs => nfs_copy_backup(file_path, config)?,
    }
    log::info!("Upload finished!");
    Ok(())
}
