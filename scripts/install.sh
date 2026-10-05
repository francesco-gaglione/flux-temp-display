#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="francesco-gaglione/flux-temp-display"
ASSET="flux-temp-display-x86_64-unknown-linux-gnu.tar.gz"
GROUP="antec-display"
RULE_NAME="99-antec-flux-pro-display.rules"
SERVICE_NAME="flux-temp-display.service"

fail() {
    echo "Error: $*" >&2
    exit 1
}

[[ "$(uname -s)" == Linux ]] || fail "This installer supports Linux only."
[[ "$(uname -m)" == x86_64 ]] || fail "Only x86_64 is currently supported."
[[ -n "${HOME:-}" && "$HOME" != "/root" ]] || fail "Run this installer as the user who will run the service, not as root."

for command in curl tar systemctl sudo install; do
    command -v "$command" >/dev/null 2>&1 || fail "Required command not found: $command"
done

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

download_url="https://github.com/${REPOSITORY}/releases/latest/download/${ASSET}"
echo "Downloading latest release..."
curl --fail --location --silent --show-error "$download_url" -o "$tmpdir/$ASSET" \
    || fail "Could not download the release asset. Has a GitHub release been published?"
tar -xzf "$tmpdir/$ASSET" -C "$tmpdir" flux_temp_display
[[ -f "$tmpdir/flux_temp_display" ]] || fail "Release archive does not contain flux_temp_display."

install -Dm755 "$tmpdir/flux_temp_display" "$HOME/.local/bin/flux_temp_display"

for file in "$RULE_NAME" "$SERVICE_NAME"; do
    if [[ "$file" == "$RULE_NAME" ]]; then
        source_path="packaging/$RULE_NAME"
    else
        source_path="packaging/$SERVICE_NAME"
    fi
    curl --fail --location --silent --show-error \
        "https://raw.githubusercontent.com/${REPOSITORY}/main/${source_path}" \
        -o "$tmpdir/$file"
done

echo "Configuring USB permissions (sudo may prompt for your password)..."
sudo groupadd --system --force "$GROUP"
sudo usermod -aG "$GROUP" "$(id -un)"
sudo install -D -m 0644 "$tmpdir/$RULE_NAME" "/etc/udev/rules.d/${RULE_NAME}"
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=usb

install -Dm644 "$tmpdir/$SERVICE_NAME" "$HOME/.config/systemd/user/${SERVICE_NAME}"
systemctl --user daemon-reload
systemctl --user enable --now "$SERVICE_NAME"

cat <<EOF
Installed and started ${SERVICE_NAME}.

If the display is not accessible immediately, log out and back in, then unplug
and reconnect it. Service logs: journalctl --user -u ${SERVICE_NAME} -f
EOF
