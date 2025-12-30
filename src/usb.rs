use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub name: String,
    pub size: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub tran: Option<String>,
    pub mountpoint: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub hotplug: Option<bool>,
    // Children partitions
    pub children: Option<Vec<Device>>,
}

pub fn get_usb_devices() -> Result<Vec<Device>> {
    let output = Command::new("lsblk")
        .args(&[
            "-J",
            "-o",
            "NAME,SIZE,TYPE,TRAN,MOUNTPOINT,VENDOR,MODEL,HOTPLUG",
        ])
        .output()
        .context("Failed to execute lsblk")?;

    if !output.status.success() {
        anyhow::bail!("lsblk failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
        .context("Failed to parse lsblk output")?;

    let usb_devices: Vec<Device> = parsed
        .blockdevices
        .into_iter()
        .filter(|d| d.tran.as_deref() == Some("usb"))
        .collect();

    Ok(usb_devices)
}

pub fn list_usbs() -> Result<()> {
    let devices = get_usb_devices()?;
    if devices.is_empty() {
        println!("No USB devices found.");
        return Ok(());
    }

    println!("{:<10} {:<10} {:<10} {:<20} {:<20}", "NAME", "SIZE", "HOTPLUG", "VENDOR", "MODEL");
    for dev in devices {
        let hotplug_str = match dev.hotplug {
            Some(true) => "YES",
            Some(false) => "NO",
            None => "-",
        };
        println!(
            "{:<10} {:<10} {:<10} {:<20} {:<20}",
            dev.name,
            dev.size,
            hotplug_str,
            dev.vendor.as_deref().unwrap_or("-"),
            dev.model.as_deref().unwrap_or("-")
        );
    }
    Ok(())
}

pub fn list_partitions(device_name: &str) -> Result<()> {
    let devices = get_usb_devices()?;
    let device = devices.iter().find(|d| d.name == device_name).with_context(|| format!("Device {} not found or is not a USB device", device_name))?;

    println!("Partitions for {}:", device_name);
    if let Some(children) = &device.children {
        println!("{:<10} {:<10} {:<10} {:<20}", "NAME", "SIZE", "TYPE", "MOUNTPOINT");
        for child in children {
            println!(
                "{:<10} {:<10} {:<10} {:<20}",
                child.name,
                child.size,
                child.device_type,
                child.mountpoint.as_deref().unwrap_or("-")
            );
        }
    } else {
        println!("No partitions found.");
    }

    Ok(())
}

pub fn sync_device(device_name: &str) -> Result<()> {
    println!("Syncing device {}...", device_name);
    // In Linux, 'sync' flushes all buffers. There isn't a per-device sync command easily accessible 
    // without valid file descriptors or using sg_utils.
    // For simplicity, we are running global sync, or we could try to sync specifically if we had a mountpoint.
    // But the user requested "sync and unmount a usb".
    // Let's run the global 'sync' command for safety.
    let status = Command::new("sync").status().context("Failed to run sync")?;
    if !status.success() {
        anyhow::bail!("sync command failed");
    }
    println!("Sync completed.");
    Ok(())
}

pub fn unmount_device(mountpoint: &str) -> Result<()> {
    println!("Unmounting {}...", mountpoint);
    let status = Command::new("umount")
        .arg(mountpoint)
        .status()
        .context("Failed to run umount")?;

    if !status.success() {
        anyhow::bail!("umount command failed");
    }
    println!("Unmounted successfully.");
    Ok(())
}

use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;
use std::fs::{self, File};
use std::io::{Read, Write};

pub fn copy_to_usb(source: &Path, dest: &Path) -> Result<()> {
    println!("Calculating size...");
    
    let mut total_size = 0;
    if source.is_file() {
        total_size = source.metadata()?.len();
    } else {
        for entry in WalkDir::new(source) {
            let entry = entry.context("Failed to read directory entry")?;
            if entry.metadata()?.is_file() {
                total_size += entry.metadata()?.len();
            }
        }
    }

    println!("Total size: {} bytes", total_size);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

    if source.is_file() {
        let file_name = source.file_name().context("Invalid source file name")?;
        let dest_path = if dest.is_dir() {
            dest.join(file_name)
        } else {
            dest.to_path_buf()
        };
        
        copy_file_with_progress(source, &dest_path, &pb)?;
    } else {
        // Directory copy
         // If dest is a dir that exists, we probably want to copy source INTO it (like cp -r)
         // But if user selected a partition mountpoint (root), we might copy source dir logic.
         // Let's assume dest is the target parent or exact target. 
         // Standard 'cp -r src dst' where dst exists -> src is copied inside dst.
         
         let file_name = source.file_name().context("Invalid source dir name")?;
         let target_root = if dest.is_dir() {
             dest.join(file_name)
         } else {
             // If dest doesn't exist, we create it as the new dir name
             dest.to_path_buf()
         };

         fs::create_dir_all(&target_root).context("Failed to create destination directory")?;

        for entry in WalkDir::new(source) {
            let entry = entry.context("Failed to read directory entry")?;
            let entry_path = entry.path();
            
            // Calculate relative path
            let relative_path = entry_path.strip_prefix(source)?;
            let dest_path = target_root.join(relative_path);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path).context("Failed to create directory")?;
            } else {
                copy_file_with_progress(entry_path, &dest_path, &pb)?;
            }
        }
    }

    pb.finish_with_message("Copy complete");
    Ok(())
}

fn copy_file_with_progress(source: &Path, dest: &Path, pb: &ProgressBar) -> Result<()> {
    let mut file_in = File::open(source).context(format!("Failed to open source file {:?}", source))?;
    let mut file_out = File::create(dest).context(format!("Failed to create dest file {:?}", dest))?;
    
    let mut buffer = [0u8; 8192];
    loop {
        let n = file_in.read(&mut buffer).context("Failed to read from file")?;
        if n == 0 {
             break;
        }
        file_out.write_all(&buffer[..n]).context("Failed to write to file")?;
        pb.inc(n as u64);
    }
    Ok(())
}
