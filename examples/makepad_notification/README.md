# Makepad Shell Notification Example

This example demonstrates macOS system notifications. **It is recommended to package and install
the app before testing**, otherwise notifications may not appear.

## Prerequisites
- Xcode Command Line Tools (`xcode-select --install`)
- cargo-packager (requires Rust 1.79+)
- robius-packaging-commands

Install:
```sh
cargo +stable install --force --locked cargo-packager
cargo install --force --locked robius-packaging-commands
```

## Package & Install (Recommended)
From this directory:
```sh
cargo packager -fdmg --release
```

The output is in `./dist`. Open the `.dmg`, drag the app into Applications, then run it.

On first run, allow notifications in **System Settings → Notifications**.

## Run
Open the app and click “Send notification”.
For demo purposes, repeated clicks send different content so notifications will show each time.
