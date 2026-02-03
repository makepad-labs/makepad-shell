# Makepad Shell Notification Example

This example demonstrates macOS system notifications. **It is recommended to package and install
the app before testing**, otherwise notifications may not appear.

## Two Ways to Run

### Option 1: Repackage After Changes (Recommended)
From this directory:
```sh
cargo packager -fdmg --release
```

The output is in `./dist`. Open the `.dmg`, drag the app into Applications, then run it.

### Option 2: Install the Prebuilt DMG (Quick Preview)
If you just want to see the demo, install the **Makepad Shell Notification** `.dmg` under `./dist`.  
This DMG is built for **Apple Silicon (M-series)**.

## Prerequisites
- Xcode Command Line Tools (`xcode-select --install`)
- cargo-packager (requires Rust 1.79+)
- robius-packaging-commands

Install:
```sh
cargo +stable install --force --locked cargo-packager
cargo install --force --locked robius-packaging-commands
```

## Usage
Open the app and click “Send notification”.  
For demo purposes, repeated clicks send different content so notifications will show each time.

On first run, allow notifications in **System Settings → Notifications**.
