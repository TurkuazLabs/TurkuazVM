# 📄 Dosya Yolu: /turkuazvm/docs/REMOTE_HOST.md
# 📌 Amac: Windows Desktop ile Linux TurkuazVM Engine remote host kurulumunu belgeler
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: TLS, token authentication, host profili, headless Engine ve systemd deployment adimlarini aciklar
# Bagimli Oldugu Katman: Controller | Service | Tool | View

# TurkuazVM Remote Host

## Guvenlik Modeli

Remote Engine API iki kosulu birlikte zorunlu tutar:

1. TLS etkin olmalidir.
2. Token authentication etkin olmalidir.

Loopback disi bir `engine_api.bind_ip` bu iki kosuldan biri eksikse Engine startup config validation tarafindan reddedilir. QMP her durumda loopback'te kalir ve LAN'a acilmaz.

Token config YAML icine yazilmaz. `token_file` veya Linux Engine tarafinda `TURKUAZVM_ENGINE_TOKEN` environment variable kullanilir. `secrets/` repository disindadir.

## Linux Engine Hazirlama

Release Engine build:

```text
cargo build --release -p turkuazvm-engine
```

Development/lab credential olusturma:

```text
./scripts/generate_remote_credentials.sh turkuazvm-engine 192.168.1.50
```

Bu script su dosyalari olusturur:

```text
secrets/remote-token.txt
secrets/tls/server-cert.pem
secrets/tls/server-key.pem
```

Production ortaminda self-signed development sertifikasi yerine kurum CA veya yonetilen bir PKI kullanilmasi onerilir.

Remote config baslangic sablonu:

```text
config/turkuazvm.remote-engine.example.yml
```

Headless baslatma:

```text
./scripts/run_engine_headless.sh /path/to/turkuazvm.yml
```

## Windows Desktop Profili

Desktop remote host ornegi:

```text
config/turkuazvm.desktop-remote.example.yml
```

Linux Engine'deki token dosyasinin ayni degeri Windows Desktop tarafinda ayri bir secret dosyasina kopyalanir. Server sertifikasi veya sertifikayi imzalayan CA sertifikasi da Desktop makinesine kopyalanir.

Remote host profilinde:

```text
mode: remote
auto_start: false
authentication.mode: token
tls.enabled: true
```

zorunludur.

Desktop host seciciden `Local Windows` veya `Linux Lab` secilebilir. Secim sonrasi ayni Tauri Controller ve DesktopService use-case'leri yeni Engine endpoint'ine yonelir.

## Network Siniri

Remote API portu yalniz guvenilen LAN/VPN aglarina firewall ile acilmalidir. QMP portu kesinlikle remote aga acilmaz. VM guest bridge/TAP network'u ile Engine management API network'u birbirinden ayri sorumluluklardir.

## systemd

Sablonlar:

```text
deploy/systemd/turkuazvm-engine.service
deploy/systemd/engine.env.example
```

Unit sablonundaki path ve service kullanicisi deployment ortamina gore ayarlanmalidir. Engine data ve runtime klasorleri service kullanicisi tarafindan yazilabilir olmalidir.

## Request Timeout Profili

Remote ve local Desktop host kayitlari iki timeout tasir. `request_timeout_ms` Dashboard/List gibi hizli Engine API actionlari icindir. `long_request_timeout_ms` Android first-boot, VM stop, snapshot/clone ve APK install gibi uzun sureli operasyonlar icindir. v0.12.3 varsayilan orneklerde 180000 ms kullanir; bu deger normal request timeoutundan kucuk olamaz ve en az 30000 ms olmalidir.
