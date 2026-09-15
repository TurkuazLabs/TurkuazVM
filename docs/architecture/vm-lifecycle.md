# 📄 Dosya Yolu: /turkuazvm/docs/architecture/vm-lifecycle.md
# 📌 Amac: TurkuazVM VM aggregate, state machine ve hypervisor lifecycle akisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Create, start, stop, repository ve QEMU process sinirlarini kilitler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool

# VM Lifecycle

## Ana akis

Controller -> VmLifecycleService -> VirtualMachine -> VmRepositoryPort / HypervisorRuntimePort

QEMU adapteri core tarafinda gorunmez. QEMU process ve QMP detaylari HypervisorRuntimePort arkasinda kalir.

## State machine

Created -> Stopped -> Starting -> Running -> Stopping -> Stopped

Ek gecisler:

- Starting -> Error
- Running -> Pausing -> Paused
- Paused -> Resuming -> Running
- Running -> Error
- Paused -> Error
- Stopping -> Error
- Error -> Stopped

Stopped -> Running gibi state atlamalari yasaktir.

## Start transaction davranisi

1. VM repository'den okunur.
2. State Starting yapilir ve kaydedilir.
3. HypervisorRuntimePort.start cagrilir.
4. Runtime basarili ise state Running yapilir.
5. Running persistence basarisiz olursa hypervisor stop compensation denenir.
6. Hypervisor start basarisiz olursa VM Error state'e alinmaya calisilir.

## QEMU process lifecycle

QemuRuntimeTool:

1. Lokal ephemeral QMP TCP endpoint ayirir.
2. QemuCommandBuilder ile launch argumanlarini uretir.
3. QEMU process'i baslatir.
4. QMP endpoint hazir olana kadar settings uzerinden kontrollu retry yapar.
5. QMP greeting + capabilities + introspection handshake tamamlaninca process registry'ye kaydeder.
6. Stop isteginde QMP quit kullanir.
7. Shutdown timeout asilirsa child process kill fallback uygular.

QMP quit cevabi gelmeden socket kapanmasi protokol tarafinda beklenebilir bir davranistir; QmpClientTool bunu basarili termination olarak yorumlar.

## Persistence

v0.1.2 ile gelen InMemoryVmRepository lifecycle test/dikey kesit adapteridir; v0.1.3 ile kalici YamlVmRepository eklenmistir.

Kalici persistence YamlVmRepository adapteri ile eklenmistir. Domain ve Service serde/YAML detaylarini bilmez.
