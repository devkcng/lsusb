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
                if let Err(e) = usb::list_usbs() {
                    println!("Error: {}", e);
                }
                wait_user();
            }
            1 => {
                // List Partitions
                match usb::get_usb_devices() {
                    Ok(devices) => {
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
                        
                        if let Err(e) = usb::list_partitions(&device_names[selection]) {
                            println!("Error: {}", e);
                        }
                    }
                    Err(e) => println!("Error listing devices: {}", e),
                }
                wait_user();
            }
            2 => {
                // Sync
                match usb::get_usb_devices() {
                    Ok(devices) => {
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
                        
                        if let Err(e) = usb::sync_device(&device_names[selection]) {
                            println!("Error: {}", e);
                        }
                    }
                    Err(e) => println!("Error listing devices: {}", e),
                }
                wait_user();
            }
            3 => {
                 // Unmount
                 match usb::get_usb_devices() {
                     Ok(devices) => {
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

                        if let Err(e) = usb::unmount_device(&mountpoints[selection]) {
                            println!("Error: {}", e);
                        }
                     }
                     Err(e) => println!("Error listing devices: {}", e),
                 }
                wait_user();
            }
            4 => {
                 // Copy
                 let source: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter path to source file/directory")
                    .interact_text()?;

                // Select destination partition
                 match usb::get_usb_devices() {
                     Ok(devices) => {
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

                        if let Err(e) = usb::copy_to_usb(&PathBuf::from(source), &final_dest) {
                            println!("Error: {}", e);
                        }
                     }
                     Err(e) => println!("Error listing devices: {}", e),
                 }
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
