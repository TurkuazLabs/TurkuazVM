#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/scripts/run_engine_headless.sh
# 📌 Amac: Linux TurkuazVM Engine'i secilen config ile headless olarak baslatir
# 📌 Modul - Bash
# Version: 0.32.0
# Aciklama: TURKUAZVM_CONFIG ayarlayip portable QEMU Turkuaz NAT destekli release Engine binary'sini foreground calistirir
# Bagimli Oldugu Katman: Tool

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_PATH="${1:-${PROJECT_ROOT}/config/turkuazvm.remote-engine.example.yml}"
ENGINE_BINARY="${PROJECT_ROOT}/target/release/turkuazvm-engine"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Config bulunamadi: ${CONFIG_PATH}"
  exit 1
fi

if [[ ! -x "${ENGINE_BINARY}" ]]; then
  echo "Engine binary bulunamadi: ${ENGINE_BINARY}"
  echo "Once cargo build --release -p turkuazvm-engine calistir."
  exit 1
fi

export TURKUAZVM_CONFIG="${CONFIG_PATH}"
exec "${ENGINE_BINARY}"
