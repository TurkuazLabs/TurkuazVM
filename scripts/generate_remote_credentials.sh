#!/usr/bin/env bash
# 📄 Dosya Yolu: /turkuazvm/scripts/generate_remote_credentials.sh
# 📌 Amac: Remote Engine icin token ve TLS development credential dosyalarini olusturur
# 📌 Modul - Bash
# Version: 0.28.0
# Aciklama: OpenSSL ile 256-bit token ve SAN iceren self-signed TLS sertifikasi uretir
# Bagimli Oldugu Katman: Tool

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SECRET_ROOT="${PROJECT_ROOT}/secrets"
TLS_ROOT="${SECRET_ROOT}/tls"
SERVER_NAME="${1:-turkuazvm-engine}"
SERVER_IP="${2:-}"
TOKEN_FILE="${SECRET_ROOT}/remote-token.txt"
CERT_FILE="${TLS_ROOT}/server-cert.pem"
KEY_FILE="${TLS_ROOT}/server-key.pem"

command -v openssl >/dev/null 2>&1 || {
  echo "OpenSSL bulunamadi."
  exit 1
}

umask 077
mkdir -p "${TLS_ROOT}"
openssl rand -hex 32 > "${TOKEN_FILE}"

SAN="DNS:${SERVER_NAME}"
if [[ -n "${SERVER_IP}" ]]; then
  SAN="${SAN},IP:${SERVER_IP}"
fi

openssl req \
  -x509 \
  -newkey rsa:3072 \
  -sha256 \
  -days 825 \
  -nodes \
  -subj "/CN=${SERVER_NAME}" \
  -addext "subjectAltName=${SAN}" \
  -keyout "${KEY_FILE}" \
  -out "${CERT_FILE}"

chmod 600 "${TOKEN_FILE}" "${KEY_FILE}"
chmod 644 "${CERT_FILE}"

echo "Remote credentials olusturuldu:"
echo "  Token: ${TOKEN_FILE}"
echo "  Cert : ${CERT_FILE}"
echo "  Key  : ${KEY_FILE}"
