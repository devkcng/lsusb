use crate::usb;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};
use std::path::PathBuf;

pub fn run() -> Result<()> {
    loop {
        let options = &[
            "List USB Devices",
            "List Partitions",
            "Sync Device",
            "Unmount Device",
            "Copy File/Dir",
            "Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action")
            .default(0)
            .items(&options[..])
            .interact()?;

        match selection {
            0 => {
                usb::list_usbs()?;
                wait_user();
            }
            1 => {
                // List Partitions
                let devices = usb::get_usb_devices()?;
                if devices.is_empty() {
                    println!("No USB devices found.");
                    wait_user();
                    continue;
                }
                let device_names: Vec<String> = devices.iter().map(|d| d.name.clone()).collect();
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a device")
                    .items(&device_names)
                    .interact()?;
                
                usb::list_partitions(&device_names[selection])?;
                wait_user();
            }
            2 => {
                // Sync
                let devices = usb::get_usb_devices()?;
                if devices.is_empty() {
                    println!("No USB devices found.");
                    wait_user();
                    continue;
                }
                 let device_names: Vec<String> = devices.iter().map(|d| d.name.clone()).collect();
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a device to sync")
                    .items(&device_names)
                    .interact()?;
                
                usb::sync_device(&device_names[selection])?;
                wait_user();
            }
            3 => {
                 // Unmount
                // Ideally this should list mountpoints. For now, let's ask for input or assume partitions.
                // To make it user friendly, let's list partitions with mountpoints.
                 let devices = usb::get_usb_devices()?;
                 let mut mountpoints = Vec::new();
                 for dev in &devices {
                     if let Some(children) = &dev.children {
                         for child in children {
                             if let Some(mp) = &child.mountpoint {
                                 mountpoints.push(mp.clone());
                             }
                         }
                     }
                 }
                
                if mountpoints.is_empty() {
                    println!("No mounted partitions found on USB devices.");
                    wait_user();
                    continue;
                }

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a mountpoint to unmount")
                    .items(&mountpoints)
                    .interact()?;

                usb::unmount_device(&mountpoints[selection])?;
                wait_user();
            }
            4 => {
                 // Copy
                 let source: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter path to source file/directory")
                    .interact_text()?;

                // Select destination partition
                 let devices = usb::get_usb_devices()?;
                 let mut mountpoints = Vec::new();
                 for dev in &devices {
                     if let Some(children) = &dev.children {
                         for child in children {
                             if let Some(mp) = &child.mountpoint {
                                 mountpoints.push(mp.clone());
                             }
                         }
                     }
                 }
                 
                 if mountpoints.is_empty() {
                     println!("No mounted partitions found. Cannot copy.");
                     wait_user();
                     continue;
                 }

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select destination partition")
                    .items(&mountpoints)
                    .interact()?;
                
                let dest_root = PathBuf::from(&mountpoints[selection]);
                // We might want to allow specifying subpath, but simple copy to root is fine for now.
                // Or we can let user type destination path inside.
                
                let use_root = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Copy to root of {}?", mountpoints[selection]))
                    .default(true)
                    .interact()?;

                let final_dest = if use_root {
                    dest_root
                } else {
                     let subpath: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter subdirectory/filename in destination")
                        .interact_text()?;
                    dest_root.join(subpath)
                };

                usb::copy_to_usb(&PathBuf::from(source), &final_dest)?;
                wait_user();

            }
            5 => break,
            _ => break,
        }
    }
    Ok(())
}

fn wait_user() {
    println!("\nPress Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());
}
