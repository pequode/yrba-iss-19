// not good enough at rust to bind to a nfs client like libnfs so going at the system bin instead. 
use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use nfs3_client::nfs3_types::xdr_codec::Opaque;
use url::Url;

use crate::upload::utils::file_name::{generate_backup_name, get_backup_name_stem};
use crate::upload::utils::backup_removal::get_all_backups_older_than_n_newest_backups;

use std::fs;
use std::path::{Path};
use std::process::Command;
use nfs3_client::tokio::TokioConnector;
use nfs3_client::{ Nfs3ConnectionBuilder};
use nfs3_client::nfs3_types::nfs3;
use tokio::runtime::Runtime;
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
    let output = Command::new("which")
        .arg("mount.nfs")
        .output()
        .context("Failed to run `which mount.nfs` as root (run yrba as sudo)")?;

    if output.status.success() {
        println!("nfs is installed!");
        Ok(())
    } else {
        Err(anyhow!(
            "mount.nfs not found. Make sure you have nfs-utils installed."
        ))
    }
}

pub fn copy_to_nfs(
    backup_file_path: &Path,
    remote_path: &str,
    ip: &str,
    config: &Config,
) -> Result<()> {

    // create a new Tokio runtime for async operations
    let rt = Runtime::new()
        .context("Failed to create Tokio runtime")?;

    // run async block for NFS client
    rt.block_on(async {
        let mut client = Nfs3ConnectionBuilder::new(TokioConnector, ip, remote_path).;

        // read the root directory (example, optional)
        let root = client.root_nfs_fh3();

        let readdir = client.readdir(&nfs3::READDIR3args {
            dir: root.clone(),
            cookie: 0,
            cookieverf: nfs3::cookieverf3::default(),
            count: 128 * 1024 * 1024,
        }).await.context("Failed to read NFS directory")?;

        println!("NFS root readdir: {:?}", readdir);

        // copy file to the "NFS root" using nfs3_client
        let backup_stem_name = get_backup_name_stem(backup_file_path)?;
        let file_name = generate_backup_name(&backup_stem_name.as_ref())?;
        let local_data = fs::read(&backup_file_path)?;
        let write_args = nfs3::WRITE3args {
            file: nfs3::nfs_fh3 { data: Opaque::from_vec(file_name.into_bytes()) },
            offset: 0,
            count: local_data.len() as u32,
            stable: nfs3::stable_how::FILE_SYNC,
            data: Opaque::from_vec(local_data),
        };
        // write a remote_file
        let write_result = client.write(&write_args);
        
       


        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
fn get_all_backups_older_than_n_newest_backups_nfs(
    n: u16,
    client:&mut Nfs3ConnectionBuilder<TokioConnector>,
    backup_stem_name: &str,
    backup_file_path: &Path
) -> anyhow::Result<Vec<&str>>{
     return Ok((vec![].iter()))
    }
fn remove_from_nfs(client: &mut Nfs3ConnectionBuilder<TokioConnector>,root:nfs3::nfs_fh3,path: &str)->Result<()>{
     
    // need to itter over dir 
    let rem_args =nfs3::REMOVE3args {
        object: nfs3::diropargs3 {
            dir: root.clone(),
            name: nfs3::filename3(Opaque::from_vec(path.as_bytes().to_vec())),
        }
    };

    
    return client.remove(&rem_args);
}
/// Remove old backup files
fn delete_n_old_backups_at_location(
    backup_stem_name: &str,
    target_location: &Path,
    config: &Config,
) -> Result<()> {
    let n = config.amount_of_backups_to_keep;
    

    let backups =
        get_all_backups_older_than_n_newest_backups_nfs(n,client,root, backup_stem_name, target_location)?;

    for backup in backups {
        if let Err(err) = remove_from_nfs(clint,root,backup.path()) {
            log::warn!(
                "Could not delete old backup at {}: {}",
                backup.path().display(),
                err
            );
        }
    }

    Ok(())
}