#!/usr/bin/env bash
set -euo pipefail

GROUP="antec-display"
RULE_NAME="99-antec-flux-pro-display.rules"
RULE_SOURCE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../packaging" && pwd)/${RULE_NAME}"
RULE_DESTINATION="/etc/udev/rules.d/${RULE_NAME}"

if [[ ${EUID} -ne 0 ]]; then
    echo "Run this installer with sudo: sudo $0 [username]" >&2
    exit 1
fi

TARGET_USER="${1:-${SUDO_USER:-}}"
if [[ -z "${TARGET_USER}" || "${TARGET_USER}" == "root" ]]; then
    echo "Specify the desktop user to grant USB access to:" >&2
    echo "  sudo $0 <username>" >&2
    exit 1
fi

if ! id "${TARGET_USER}" >/dev/null 2>&1; then
    echo "User '${TARGET_USER}' does not exist." >&2
    exit 1
fi

if [[ ! -f "${RULE_SOURCE}" ]]; then
    echo "udev rule not found: ${RULE_SOURCE}" >&2
    exit 1
fi

if ! getent group "${GROUP}" >/dev/null; then
    groupadd --system "${GROUP}"
fi

usermod -aG "${GROUP}" "${TARGET_USER}"
install -D -m 0644 "${RULE_SOURCE}" "${RULE_DESTINATION}"
udevadm control --reload-rules
udevadm trigger --subsystem-match=usb

cat <<EOF
Installed the Antec Flux Pro USB permissions rule.
User '${TARGET_USER}' was added to group '${GROUP}'.

Log out and back in for the new group membership to take effect, then unplug
and reconnect the display. Verify it is detected with:
  lsusb -d 2022:0522
EOF
