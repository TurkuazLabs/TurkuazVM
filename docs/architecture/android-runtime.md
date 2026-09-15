# 📄 Dosya Yolu: /turkuazvm/docs/architecture/android-runtime.md
# 📌 Amac: Android Runtime bounded context dependency ve use-case mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Android domain, Engine orchestration, ADB adapter, persistence ve network provisioning sinirlarini kilitler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# Android Runtime Architecture

## Ana Kural

Android Runtime normal VM domain'inin icine ADB, APK veya package manager mantigi eklemez.

```text
Desktop View
    -> Tauri Controller
    -> DesktopService
    -> Engine API v5
    -> EngineApplicationService
    -> AndroidApplicationService
        |-> VmQueryService
        |-> NetworkService
        |-> AndroidRuntimeService
        |-> AndroidPackageService
        `-> AndroidInputService
               |
               v
          Android Ports
          /           \
YamlAndroidRepo     AdbRuntimeTool
```

## Bounded Context Siniri

`crates/android`:

- Android runtime profile domain
- Device state/ABI domain
- Package domain
- Input action domain
- Command DTO'lari
- Port contractlari
- Application/domain services

Bilmez:

- QEMU command line
- QMP
- Tauri
- serde/YAML
- `std::process::Command`

`crates/guest` icindeki `AdbRuntimeTool`, Android portlarini gercek ADB process cagirilariyla uygular.

## Provisioning Transaction

```text
ConfigureAndroidRuntime
    -> Require Android VM
    -> Require Stopped
    -> Read existing android.yml
    -> Validate/allocate loopback ADB port
    -> Validate or attach android-nat
    -> Persist android.yml
```

Profile persistence basarisiz olur ve `android-nat` bu islemde yeni olusturulmussa `DetachNetworkCommand` compensation calisir.

Mevcut `android-nat` ile `android.yml` farkli host ADB portlari isaret ediyorsa islem fail-fast olur; sessizce bozuk runtime profili uretilmez.

## Persistence

`YamlAndroidProfileRepository` Android bounded context adapteridir.

```text
machines/<vm-id>/android.yml
```

schema version: `1`

Core `machine.yml` schema version 5 olarak kalir.

## Package Path Security

PackageService iki asamali kontrol uygular:

1. Relative `.apk` lexical validation.
2. Canonical root + canonical candidate containment validation.

Bu nedenle `../`, absolute path ve symlink ile package root disina cikis kabul edilmez.

## ADB Adapter

ADB adapteri:

- explicit binary config
- Android SDK environment path
- PATH

sirasi ile binary arar.

ADB process output'u bounded temp dosyalarina yonlendirilir. Process timeout durumunda child sonlandirilir ve temp dosyalari temizlenir.

## Gaming'e Gecis

v0.6.0 control-plane foundation'dir. v0.7.0 GPU data-plane eklendiginde Android service contractlari korunur; ADB renderer yerine kullanilmaz.
