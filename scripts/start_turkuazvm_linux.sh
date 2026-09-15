#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/scripts/start_turkuazvm_linux.sh
# 📌 Amac: Linux hostta TurkuazVM QEMU, Rust ve Desktop gereksinimlerini dogrulayip uygulamayi baslatir
# 📌 Modul - Bash
# Version: 0.37.0
# Aciklama: OpenVPN/TAP gerektirmeyen QEMU User NAT varsayilaniyla Engine, Display ve Tauri Desktop akisini calistirir
# Bagimli Oldugu Katman: Tool | View

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_PATH="${TURKUAZVM_CONFIG:-${PROJECT_ROOT}/config/turkuazvm.yml}"
WORKSPACE_VERSION="$(awk '/^\[workspace.package\]/{in_pkg=1;next} in_pkg && /^\[/{exit} in_pkg && /^version[[:space:]]*=/{gsub(/["[:space:]]/, "", $2); print $2; exit}' "${PROJECT_ROOT}/Cargo.toml")"
MIN_RUST_VERSION="1.98.0"

status() {
  printf '%-28s %s\n' "$1" "$2"
}

require_command() {
  local name="$1"
  if command -v "${name}" >/dev/null 2>&1; then
    status "${name}" "OK"
    return 0
  fi
  status "${name}" "NOT_FOUND"
  return 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "LINUX_REQUIRED: Bu launcher Linux host icindir."
  exit 1
fi

cd "${PROJECT_ROOT}"

echo ""
echo "TurkuazVM Launcher v${WORKSPACE_VERSION} - Linux Preflight"
echo ""

failures=0
require_command rustc || failures=$((failures + 1))
require_command cargo || failures=$((failures + 1))
if command -v rustc >/dev/null 2>&1; then
  rust_version="$(rustc --version | awk '{print $2}')"
  if [[ "$(printf '%s\n%s\n' "${MIN_RUST_VERSION}" "${rust_version}" | sort -V | head -n1)" != "${MIN_RUST_VERSION}" ]]; then
    status "Rust version" "TOO_OLD ${rust_version} < ${MIN_RUST_VERSION}"
    failures=$((failures + 1))
  else
    status "Rust version" "OK ${rust_version}"
  fi
fi
require_command qemu-system-x86_64 || failures=$((failures + 1))
require_command qemu-img || failures=$((failures + 1))
require_command pkg-config || failures=$((failures + 1))

if [[ -r /dev/kvm && -w /dev/kvm ]]; then
  status "KVM" "OK"
else
  status "KVM" "OPTIONAL_NOT_READY_TCG_FALLBACK"
fi

if [[ -f "${CONFIG_PATH}" ]]; then
  status "config/turkuazvm.yml" "OK"
else
  status "config/turkuazvm.yml" "NOT_FOUND"
  failures=$((failures + 1))
fi

status "Turkuaz NAT Runtime" "QEMU_USER_NAT"
status "OpenVPN / TAP" "NOT_REQUIRED"

if (( failures > 0 )); then
  echo ""
  echo "BASLATMA ENGELLENDI"
  echo "Eksik Linux paketlerini dagitiminizin paket yoneticisiyle kurup tekrar calistirin."
  echo "Tauri Desktop icin WebKitGTK 4.1 ve sistem GUI gelistirme paketleri de gerekli olabilir."
  exit 1
fi

export TURKUAZVM_CONFIG="${CONFIG_PATH}"

echo ""
echo "Engine ve Display derleniyor..."
cargo build -p turkuazvm-engine -p turkuazvm-display

echo ""
echo "TurkuazVM Desktop baslatiliyor..."
exec cargo run -p turkuazvm-desktop
