# 📄 Dosya Yolu: /turkuazvm/docs/architecture/desktop-foundation.md
# 📌 Amac: TurkuazVM v0.2.0 Desktop ve local Engine API mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Tauri shell, versioned Engine API, process bootstrap ve runtime update sinirlarini kilitler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# Desktop Foundation

TurkuazVM Desktop QEMU'yu, repository dosyalarini veya platform network adapterlerini dogrudan cagiramaz. Desktop yalniz versioned Engine API contractini bilir.

## Dependency akisi

```text
Vanilla View
    |
    v
Tauri Controller
    |
    v
DesktopService
    |
    v
EngineApiClientTool
    |
    | localhost JSON line protocol
    v
EngineApiServerTool
    |
    v
EngineApiController
    |
    v
EngineApplicationService
    |
    +--> HostCapabilityService
    +--> VmQueryService
    `--> VmLifecycleService
              |
              +--> SharedVmRepository
              +--> NativeNetworkTool
              `--> EngineHypervisorRuntimeTool
                         |
                         `--> QemuRuntimeTool
```

## Process siniri

Desktop ve Engine ayri process'tir. Desktop UI kapansa bile Engine process model olarak bagimsiz kalabilir. Bu sinir v0.4.0 Remote Host hedefinin temelidir.

`crates/engine-api` yalniz transport DTO ve API version bilgisini tasir. Core bu crate'i import etmez.

## Local API

v0.2.0 local API newline-delimited JSON uzerinden loopback TCP kullanir. Her connection tek request ve tek response tasir. Contract `ENGINE_API_VERSION` ile versionlanir.

Desteklenen requestler:

- ping
- dashboard
- list_vms
- create_vm
- start_vm
- stop_vm

Local API v0.2.0'da remote erisim icin tasarlanmamistir. Bind adresi config ile loopback olarak tutulur. Authenticated remote transport v0.4.0 kapsamidir.

## Desktop runtime update

UI ilk dashboard'u Tauri invoke ile alir. Desktop composition root kucuk dashboard snapshotlarini config ile belirlenen aralikta Engine'den okuyup Tauri event sistemi ile View'a yayar.

Bu kanal GPU frame veya yuksek hacimli telemetry icin kullanilmaz. Native display ve yuksek frekans telemetry ayri runtime kanali kullanacaktir.

## Engine auto-start

Desktop Engine'e baglanamazsa ve `desktop.engine.auto_start` aciksa `EngineProcessTool` config'te tanimli binary'yi baslatir. Yeni process'e ayni `TURKUAZVM_CONFIG` dosyasi aktarilir. Desktop request'i startup timeout dolana kadar retry eder.

## UI scope

v0.2.0 UI:

- Engine/QEMU/host status
- VM list
- basic create wizard
- start/stop
- runtime state update

Disk, ISO, firmware ve gelismis network wizard ekranlari sonraki Desktop minor surumlerinde mevcut core use-case'lerine baglanacaktir.

## v0.12.3 Request Timeout Siniflandirmasi

Desktop host profili hizli requestler icin `request_timeout_ms`, Android first-boot ve uzun storage/runtime islemleri icin `long_request_timeout_ms` tasir. Timeout secimi DesktopService icinde EngineAction turune gore yapilir; EngineApiClientTool yalniz verilen timeout ile TCP/TLS I/O uygular.
