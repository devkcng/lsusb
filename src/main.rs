mod cli;
mod usb;
mod tui;

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(command) => match command {
            cli::Commands::List => {
                usb::list_usbs()?;
            }
            cli::Commands::Parts { device } => {
                usb::list_partitions(&device)?;
            }
            cli::Commands::Sync { device } => {
                usb::sync_device(&device)?;
            }
            cli::Commands::Unmount { device } => {
                usb::unmount_device(&device)?;
            }
            cli::Commands::Cp { source, dest } => {
                usb::copy_to_usb(&source, &dest)?;
            }
        },
        None => {
            tui::run()?;
        }
    }

    Ok(())
}
