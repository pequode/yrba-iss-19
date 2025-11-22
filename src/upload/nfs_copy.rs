// not good enough at rust to bind to a nfs client like libnfs so going at the system bin instead. 
use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use url::Url;

use crate::upload::utils::file_name::{generate_backup_name, get_backup_name_stem};
use crate::upload::utils::backup_removal::get_all_backups_older_than_n_newest_backups;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;


pub fn nfs_copy_backup(backup_file_path: &Path, config: &Config) -> Result<()> {
    // parse URL
    let remote_url: Url = Url::parse(&config.remote)
        .context(format!("Could not parse remote URL! {}", &config.remote))?;

    let ip = get_ip_from_url(&remote_url)?;
    let remote_path = remote_url.path();

    // verify binary exists
    check_for_nfs_bin()?;

    // perform copy
    copy_to_nfs(backup_file_path, remote_path, ip, config)?;

    Ok(())
}

/// Extract host and ensure correct port
fn get_ip_from_url(url: &Url) -> Result<&str> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("Could not retrieve remote host from URL!"))?;

    let port = url.port().unwrap_or(2049);

    if port != 2049 {
        return Err(anyhow!(
            "Currently only port 2049 is supported for NFSv4 (got {})",
            port
        ));
    }

    Ok(host)
}

/// Verify `mount.nfs` exists
fn check_for_nfs_bin() -> Result<()> {
    let output = Command::new("sudo")
        .arg("which")
        .arg("mount.nfs")
        .output()
        .context("Failed to run `sudo which mount.nfs`")?;

    if output.status.success() {
        println!("nfs is installed!");
        Ok(())
    } else {
        Err(anyhow!(
            "mount.nfs not found. Make sure you have nfs-utils installed."
        ))
    }
}

fn copy_to_nfs(
    backup_file_path: &Path,
    remote_path: &str,
    ip: &str,
    config: &Config,
) -> Result<()> {
    // create temporary mount directory
    let temp_dir = TempDir::new()?;

    mount_nfs(ip, remote_path, temp_dir.path())?;

    let result = (|| {
        let backup_stem_name = get_backup_name_stem(backup_file_path)?;
        let file_name = generate_backup_name(&backup_stem_name)?;
        let target_path = temp_dir.path().join(&file_name);

        fs::copy(backup_file_path, &target_path)
            .context("Failed copying backup into NFS mount")?;

        delete_n_old_backups_at_location(&backup_stem_name, temp_dir.path(), config)?;

        Ok(())
    })();

    // always attempt to unmount
    unmount_nfs(temp_dir.path())?;

    // still return main result
    result
}

/// Mount the NFS share
fn mount_nfs(host: &str, path: &str, local_mount: &Path) -> Result<()> {
    let target = format!("{}:{}", host, path);

    let output = Command::new("sudo")
        .arg("mount")
        .arg("-t")
        .arg("nfs")
        .arg(&target)
        .arg(local_mount)
        .output()
        .context("Failed to mount NFS")?;

    if !output.status.success() {
        return Err(anyhow!(
            "mount failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Unmount the NFS folder
fn unmount_nfs(local_mount: &Path) -> Result<()> {
    let output = Command::new("sudo")
        .arg("umount")
        .arg(local_mount)
        .output()
        .context("Failed trying to unmount")?;

    if !output.status.success() {
        return Err(anyhow!(
            "unmount failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Remove old backup files
fn delete_n_old_backups_at_location(
    backup_stem_name: &str,
    target_location: &Path,
    config: &Config,
) -> Result<()> {
    let n = config.amount_of_backups_to_keep;

    let backups =
        get_all_backups_older_than_n_newest_backups(n, backup_stem_name, target_location)?;

    for backup in backups {
        if let Err(err) = fs::remove_file(backup.path()) {
            log::warn!(
                "Could not delete old backup at {}: {}",
                backup.path().display(),
                err
            );
        }
    }

    Ok(())
}