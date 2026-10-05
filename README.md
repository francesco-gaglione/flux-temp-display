# Flux Temp Display

A small Linux service that displays CPU and GPU temperatures on the built-in USB display of the **Antec Flux Pro** case (USB vendor/product ID `2022:0522`). Temperatures are read from Linux thermal and hwmon sensors. The service samples sensors once per second and refreshes the display twice per second; on a normal shutdown it clears the display.

> **Compatibility:** Linux only. The USB protocol is specific to the Antec Flux Pro display. Other cases and displays are not supported or tested.

## Features

- Displays CPU and GPU temperatures in °C.
- Supports thermal-zone and hwmon sensor sources exposed under `/sys`.
- Prefers CPU package and GPU edge/core sensors when available; hotspot sensors are fallback-only.
- On systems with multiple AMD GPUs, prefers the GPU with the largest reported VRAM.
- Runs in the foreground or as a per-user systemd service.

## Requirements

- Linux with `/sys/class/thermal` and/or `/sys/class/hwmon` sensor data.
- Rust toolchain and Cargo (edition 2024; Rust 1.85 or newer).
- USB access to the Antec display. The included udev rule sets this up for Fedora and other systemd/udev distributions.
- Build tools required by Rust dependencies. The `rusb` dependency builds its libusb backend from bundled sources.

On Fedora, install the USB utilities and development tools:

```bash
sudo dnf install usbutils gcc make
```

## Build and run

```bash
cargo test --locked
cargo build --release --locked
./target/release/flux_temp_display
```

The process stays in the foreground; press `Ctrl+C` to stop it. Sensor read failures are written to stderr and the corresponding display value is shown as unavailable. If the display is not found, check USB permissions below.

## Install from a release

On supported x86_64 Linux systems with systemd, run the installer as your normal user (not with `sudo`):

```bash
curl -fsSL https://raw.githubusercontent.com/francesco-gaglione/flux-temp-display/main/scripts/install.sh | bash
```

It downloads the latest GitHub release, installs the binary under `~/.local/bin`, installs and starts the per-user systemd service, and configures USB permissions. It uses `sudo` to create the `antec-display` group and install the udev rule. Log out and back in, then reconnect the display, if group membership changes.

Releases are built and published by GitHub Actions when a version tag is pushed. To publish one, for example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow currently builds the x86_64 Linux binary. The installer can be run again to update to the latest release.

## USB permissions

Confirm that Linux detects the display:

```bash
lsusb -d 2022:0522
```

Install the udev rule and grant access to the user that will run the service (replace `YOUR_USER`):

```bash
sudo ./scripts/install-udev.sh YOUR_USER
```

The installer creates the `antec-display` system group, adds the selected user to it, installs the udev rule, and reloads rules. Log out and back in for the new group membership to take effect, then unplug and reconnect the display. The rule grants access to members of `antec-display` and enables `uaccess` for active desktop sessions.

## Run as a systemd user service

Build and install the executable, then enable the included user unit:

```bash
cargo build --release --locked
install -Dm755 target/release/flux_temp_display "$HOME/.local/bin/flux_temp_display"
mkdir -p "$HOME/.config/systemd/user"
install -m 644 packaging/flux-temp-display.service "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now flux-temp-display.service
```

Inspect logs and stop the service with:

```bash
journalctl --user -u flux-temp-display -f
systemctl --user stop flux-temp-display
```

## Troubleshooting

- **Display not found or permission denied:** verify `lsusb -d 2022:0522`, install the udev rule, refresh group membership, and reconnect the display.
- **Temperature unavailable:** sensor names and availability depend on the kernel and hardware drivers. Check that readings exist under `/sys/class/thermal` or `/sys/class/hwmon`.
- **Service does not start:** inspect its user journal with the command above and confirm that the executable is at `~/.local/bin/flux_temp_display`.

## Development

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
