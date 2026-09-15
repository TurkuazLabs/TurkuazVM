# 📄 Dosya Yolu: /turkuazvm/docs/architecture/gaming-gpu.md
# 📌 Amac: TurkuazVM Gaming GPU katman sinirlarini ve adapter mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: GPU domain, host probe, QEMU probe, runtime strategy ve Desktop capability akisini ayirir
# Bagimli Oldugu Katman: Service | Tool | View

# Gaming GPU Architecture

```text
Desktop View
    -> DesktopService
    -> Engine API v7 (v6 GPU contract korunur)
    -> EngineApplicationService
       -> GamingGpuService
          -> HostGpuProbePort
             <- NativeGpuProbeTool
          -> HypervisorGpuProbePort
             <- QemuGpuProbeTool
       -> ResolvedGamingGpuProfile
          -> QemuGpuRuntimeSettings
             -> QemuCommandBuilder
```

## Dependency Kurali

`crates/gpu` QEMU, platform, Tauri, serde veya process API bilmez.

QEMU device/property syntax'i `crates/qemu` altinda kalir. Windows/Linux Vulkan loader discovery ise `crates/platform` altinda kalir.

## Runtime Strategy

Engine config policy'yi parse eder fakat backend uygunluguna karar vermez. `GamingGpuService` gercek capability raporuna gore policy'yi resolve eder.

QEMU adapteri yalniz resolve edilmis runtime settings alir:

```text
GpuBackend
hostmem_mib
experimental
```

## Fail Safe

Explicit backend mevcut degilse Engine process'i crash etmez. GPU resolution error dashboard/capability endpoint'inde gorunur ve runtime `software` backend'e duser.

Bu davranis VM yonetiminin GPU capability eksigi nedeniyle tamamen kullanilamaz hale gelmesini engeller.

## Windows Gaming Direction

Stock QEMU accelerated virtio-gpu yolu v0.7.0'da Linux host capability olarak ele alinir. Windows Gaming Runtime sonraki surumlerde `GamingGpuService` arkasina yeni bir hypervisor/renderer adapteri olarak eklenir.
