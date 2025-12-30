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

pub fn copy_to_usb(source: &Path, dest: &Path) -> Result<()> {
    println!("Copying {:?} to {:?}...", source, dest);
    
    // Using system 'cp' command for simplicity and robustness with recursive copying
    let status = Command::new("cp")
        .arg("-r")
        .arg(source)
        .arg(dest)
        .status()
        .context("Failed to run cp command")?;

    if !status.success() {
        anyhow::bail!("cp command failed");
    }

    println!("Copy completed successfully.");
    Ok(())
}
