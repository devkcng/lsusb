# lsusb (Rust)

A modern, Rust-based CLI tool and TUI for managing USB devices on Linux. It provides functionalities to list devices, show partitions, sync, unmount, and copy files.

## Features

- **Interactive TUI**: Easy-to-use terminal user interface for selecting actions and devices.
- **Device Listing**: Lists all connected USB storage devices (using `lsblk`).
- **Partition Details**: Shows mounted partitions for a selected device.
- **Safe Sync**: Flushes buffers to ensure data integrity before removal.
- **Unmount**: Safely unmounts device partitions.
- **File Copy**: Recursively copies files or directories to a USB device.

## Prerequisites

- **Operating System**: Linux (relies on `lsblk`, `sync`, `umount`, `cp`).
- **Dependencies**:
  - `lsblk`: Must be installed and available in standard paths.

## Installation

Ensure you have Rust and Cargo installed.

```bash
git clone <repository_url>
cd lsusb
cargo build --release
```

## Usage

### Interactive Mode (TUI)

Run without arguments to launch the interactive menu:

```bash
cargo run
# or if built
./target/release/lsusb
```

### CLI Mode

You can also use command-line arguments for scripts or direct execution.

#### List USB Devices

```bash
cargo run -- list
```

#### List Partitions

```bash
cargo run -- parts <DEVICE_NAME>
# Example: cargo run -- parts sdb
```

#### Sync Device

```bash
cargo run -- sync <DEVICE_NAME>
# Example: cargo run -- sync sdb
```

#### Unmount Device

```bash
cargo run -- unmount <MOUNTPOINT>
# Example: cargo run -- unmount /run/media/user/DISK
```

#### Copy Files

```bash
cargo run -- cp <SOURCE> <DESTINATION>
# Example: cargo run -- cp ./my_file.txt /run/media/user/DISK
```

## Dependencies

- [clap](https://crates.io/crates/clap): CLI argument parsing.
- [dialoguer](https://crates.io/crates/dialoguer): Terminal user interface.
- [serde](https://crates.io/crates/serde) & [serde_json](https://crates.io/crates/serde_json): JSON parsing.
- [anyhow](https://crates.io/crates/anyhow): Error handling.
