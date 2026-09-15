# 📄 Dosya Yolu: /turkuazvm/docs/architecture/guest-boot-foundation.md
# 📌 Amac: TurkuazVM guest boot, ISO ve firmware mimari sinirlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: BIOS/UEFI boot, installer media, boot order ve adapter sorumluluklarini belgeler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool

# Guest Boot Foundation

## Dikey akis

EngineApiController -> EngineApplicationService -> GuestBootService -> VmRepositoryPort + MediaPort + FirmwarePort

Adapterler:

- YamlVmRepository: guest boot configuration persistence.
- LocalGuestMediaTool: installer ISO dosyasini machine root altina kopyalar.
- UefiFirmwareTool: ortak UEFI code dosyasini hazirlar ve VM basina writable vars kopyasi olusturur.
- QemuCommandBuilder: domain configuration bilgisini QEMU boot argumanlarina cevirir.

## Path politikasi

Installer ISO machine root altinda relative path ile saklanir:

    data/machines/<vm-id>/media/installer.iso

UEFI code global data root altinda tek kopya olabilir:

    data/firmware/OVMF_CODE.fd

UEFI vars her VM icin ayri tutulur:

    data/machines/<vm-id>/firmware/OVMF_VARS.fd

Bu tasarim VM NVRAM state paylasimini engeller.

## Boot order

Domain boot device enumlari QEMU harflerini bilmez. QEMU adapteri x86 boot harflerine ceviri yapar:

- Disk -> c
- Cdrom -> d
- Network -> n

Installer akisi icin boot_once true kullanildiginda adapter `-boot once=dc` benzeri arguman uretir. Boylece ilk acilista CD-ROM oncelikli olur; sonraki reboot firmware varsayilan diske donebilir.

## Firmware

BIOS seciminde QEMU default x86 firmware kullanilir ve ekstra firmware argumani uretilmez.

UEFI seciminde iki pflash drive uretilir:

- code: read-only
- vars: writable ve VM-local

Core katmani OVMF, pflash veya QEMU path bilgisi bilmez.

## Display

Guest kurulumu icin QemuDisplayMode::Default eklendi. Default modda `-display none` uretilmez ve QEMU host ortaminda uygun varsayilan display backend'ini acabilir. Headless kullanim icin None korunur.
