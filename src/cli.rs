use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lsusb")]
#[command(about = "A tool to manage USB devices", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all USB devices
    List,
    /// Show partitions for a device
    Parts {
        /// The device name (e.g., sdb)
        device: String,
    },
    /// Sync a device (flush buffers)
    Sync {
        /// The device name (e.g., sdb)
        device: String,
    },
    /// Unmount a device partition
    Unmount {
        /// The mountpoint to unmount
        device: String,
    },
    /// Copy file or directory to a USB partition
    Cp {
        /// Source file or directory
        source: PathBuf,
        /// Destination path on the USB
        dest: PathBuf,
    },
}
