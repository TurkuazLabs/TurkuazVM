#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/TurkuazVM-Start.sh
# 📌 Amac: TurkuazVM Linux Desktop launcher aracini proje kokunden baslatir
# 📌 Modul - Bash
# Version: 0.35.0
# Aciklama: Portable QEMU Turkuaz NAT kullanan Linux preflight ve Desktop baslatma scriptine giris noktasi saglar
# Bagimli Oldugu Katman: Tool | View

set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${PROJECT_ROOT}/scripts/start_turkuazvm_linux.sh"
