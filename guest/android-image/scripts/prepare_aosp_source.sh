#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/guest/android-image/scripts/prepare_aosp_source.sh
# 📌 Amac: Turkuaz Android image build'i icin resmi AOSP android-latest-release kaynak agacini hazirlar
# 📌 Modul - Shell
# Version: 0.28.0
# Aciklama: Linux hostta Repo client init/sync ve minimum disk preflight islemlerini kontrollu yapar
# Bagimli Oldugu Katman: Tool | Config

set -euo pipefail

readonly MANIFEST_URL="https://android.googlesource.com/platform/manifest"
readonly SOURCE_BRANCH="android-latest-release"
readonly MINIMUM_FREE_GIB=400
readonly DEFAULT_SYNC_JOBS=8

usage() {
  echo "Usage: $0 --aosp-root <path> [--sync-jobs <count>]"
}

AOSP_ROOT=""
SYNC_JOBS="${DEFAULT_SYNC_JOBS}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --aosp-root) AOSP_ROOT="$2"; shift 2 ;;
    --sync-jobs) SYNC_JOBS="$2"; shift 2 ;;
    *) usage; exit 2 ;;
  esac
done

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: AOSP source preparation requires Linux." >&2
  exit 3
fi
if [[ -z "${AOSP_ROOT}" ]]; then
  usage
  exit 2
fi
if ! [[ "${SYNC_JOBS}" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: sync jobs must be a positive integer." >&2
  exit 4
fi
if ! command -v repo >/dev/null 2>&1; then
  echo "ERROR: Repo launcher is not available in PATH." >&2
  exit 5
fi

mkdir -p "${AOSP_ROOT}"
readonly FREE_KIB="$(df -Pk "${AOSP_ROOT}" | awk 'NR==2 {print $4}')"
readonly REQUIRED_KIB=$((MINIMUM_FREE_GIB * 1024 * 1024))
if [[ "${FREE_KIB}" -lt "${REQUIRED_KIB}" ]]; then
  echo "ERROR: AOSP build host needs at least ${MINIMUM_FREE_GIB} GiB free disk." >&2
  exit 6
fi

cd "${AOSP_ROOT}"
if [[ ! -d .repo ]]; then
  repo init --partial-clone --no-use-superproject -b "${SOURCE_BRANCH}" -u "${MANIFEST_URL}"
fi
repo sync -c -j"${SYNC_JOBS}"

printf 'AOSP source ready: %s / branch=%s\n' "${AOSP_ROOT}" "${SOURCE_BRANCH}"
