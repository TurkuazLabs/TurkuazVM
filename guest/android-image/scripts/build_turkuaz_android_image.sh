#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/guest/android-image/scripts/build_turkuaz_android_image.sh
# 📌 Amac: Turkuaz Android AOSP product overlay'ini uygular, image build eder ve TurkuazVM bundle'i olusturur
# 📌 Modul - Shell
# Version: 0.28.0
# Aciklama: android-latest-release kaynak agacinda x86_64 Cuttlefish userdebug image uretim akisini otomatiklestirir
# Bagimli Oldugu Katman: Tool | Config | Repo

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly IMAGE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly PROJECT_ROOT="$(cd "${IMAGE_ROOT}/../.." && pwd)"
readonly OVERLAY_ROOT="${IMAGE_ROOT}/aosp-overlay"
readonly AGENT_ROOT="${PROJECT_ROOT}/guest/android-agent"
readonly PRODUCT_NAME="turkuazvm_cf_x86_64_phone"
readonly RELEASE_CONFIG="aosp_current"
readonly BUILD_VARIANT="userdebug"
readonly LUNCH_TARGET="${PRODUCT_NAME}-${RELEASE_CONFIG}-${BUILD_VARIANT}"
readonly COMPOSITE_BUILDER="${SCRIPT_DIR}/assemble_turkuaz_android_composite.py"

usage() {
  echo "Usage: $0 --aosp-root <path> --output-root <path> --image-id <id>"
}

AOSP_ROOT=""
OUTPUT_ROOT=""
IMAGE_ID=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --aosp-root) AOSP_ROOT="$2"; shift 2 ;;
    --output-root) OUTPUT_ROOT="$2"; shift 2 ;;
    --image-id) IMAGE_ID="$2"; shift 2 ;;
    *) usage; exit 2 ;;
  esac
done

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: AOSP image build requires a Linux host." >&2
  exit 3
fi
if [[ -z "${AOSP_ROOT}" || -z "${OUTPUT_ROOT}" || -z "${IMAGE_ID}" ]]; then
  usage
  exit 2
fi
if [[ ! "${IMAGE_ID}" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "ERROR: Invalid image id." >&2
  exit 4
fi
if [[ ! -f "${AOSP_ROOT}/build/envsetup.sh" ]]; then
  echo "ERROR: AOSP source root is invalid: ${AOSP_ROOT}" >&2
  exit 5
fi
if [[ ! -f "${AGENT_ROOT}/Android.bp" ]]; then
  echo "ERROR: TurkuazInputAgent source is missing." >&2
  exit 6
fi

readonly TARGET_DEVICE_ROOT="${AOSP_ROOT}/device/turkuazvm"
readonly TARGET_PRODUCT_ROOT="${TARGET_DEVICE_ROOT}/cuttlefish"
readonly TARGET_AGENT_ROOT="${TARGET_DEVICE_ROOT}/input-agent"
readonly BUNDLE_ROOT="${OUTPUT_ROOT}/${IMAGE_ID}"

rm -rf "${TARGET_PRODUCT_ROOT}" "${TARGET_AGENT_ROOT}"
mkdir -p "${TARGET_PRODUCT_ROOT}" "${TARGET_AGENT_ROOT}" "${BUNDLE_ROOT}"
cp -a "${OVERLAY_ROOT}/device/turkuazvm/cuttlefish/." "${TARGET_PRODUCT_ROOT}/"
cp -a "${AGENT_ROOT}/." "${TARGET_AGENT_ROOT}/"

cd "${AOSP_ROOT}"
# shellcheck disable=SC1091
source build/envsetup.sh
lunch "${LUNCH_TARGET}"
m

readonly PRODUCT_OUT="${ANDROID_PRODUCT_OUT}"
if [[ ! -d "${PRODUCT_OUT}" ]]; then
  echo "ERROR: ANDROID_PRODUCT_OUT is unavailable after build." >&2
  exit 7
fi

rm -rf "${BUNDLE_ROOT}"
mkdir -p "${BUNDLE_ROOT}"

copy_if_present() {
  local name="$1"
  if [[ -f "${PRODUCT_OUT}/${name}" ]]; then
    cp -f "${PRODUCT_OUT}/${name}" "${BUNDLE_ROOT}/${name}"
  fi
}

copy_if_present boot.img
copy_if_present init_boot.img
copy_if_present vendor_boot.img
copy_if_present super.img
copy_if_present userdata.img
copy_if_present vbmeta.img
copy_if_present vbmeta_system.img
copy_if_present metadata.img
copy_if_present misc.img
copy_if_present kernel

if [[ -f "${PRODUCT_OUT}/bootloader" ]]; then
  cp -f "${PRODUCT_OUT}/bootloader" "${BUNDLE_ROOT}/bootloader.qemu"
elif [[ -f "${PRODUCT_OUT}/bootloader.qemu" ]]; then
  cp -f "${PRODUCT_OUT}/bootloader.qemu" "${BUNDLE_ROOT}/bootloader.qemu"
else
  echo "ERROR: Cuttlefish QEMU bootloader artifact is missing." >&2
  exit 8
fi

python3 "${COMPOSITE_BUILDER}" --product-out "${PRODUCT_OUT}" --output "${BUNDLE_ROOT}/composite.img"

for required in boot.img super.img userdata.img bootloader.qemu composite.img; do
  if [[ ! -s "${BUNDLE_ROOT}/${required}" ]]; then
    echo "ERROR: Required Android image artifact missing: ${required}" >&2
    exit 8
  fi
done

ANDROID_RELEASE="$(get_build_var PLATFORM_VERSION 2>/dev/null || true)"
ANDROID_SDK="$(get_build_var PLATFORM_SDK_VERSION 2>/dev/null || true)"
SOURCE_REVISION="$(git -C "${AOSP_ROOT}/build/make" rev-parse HEAD 2>/dev/null || true)"

cat > "${BUNDLE_ROOT}/build-info.yml" <<EOF
# 📄 Dosya Yolu: generated/android-images/${IMAGE_ID}/build-info.yml
# 📌 Amac: Turkuaz Android build provenance metadata bilgisini saklar
# 📌 Generated - YAML
# Version: 0.12.4
# Aciklama: Engine artifact registration sirasinda okunacak source revision, Android release ve SDK bilgisini tutar
# Bagimli Oldugu Katman: Repo | Tool
source_revision: "${SOURCE_REVISION}"
android_release: "${ANDROID_RELEASE}"
sdk_level: ${ANDROID_SDK:-0}
EOF

printf 'Turkuaz Android bundle ready: %s
' "${BUNDLE_ROOT}"
