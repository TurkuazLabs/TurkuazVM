# 📄 Dosya Yolu: /turkuazvm/docs/architecture/platform-networking.md
# 📌 Amac: TurkuazVM Windows/Linux host network runtime mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.32.0
# Aciklama: Portable QEMU User NAT, Advanced TAP/bridge, lease, PID binding, cleanup ve crash recovery sorumluluklarini ayirir
# Bagimli Oldugu Katman: Service | Repo | Tool

# Platform Networking

## Dependency Akisi

    VmLifecycleService
        -> NetworkPort
        -> NativeNetworkTool

    VmLifecycleService
        -> HypervisorRuntimePort
        -> QemuRuntimeTool
        -> QemuCommandBuilder

Core host komutlarini ve QEMU network syntax'ini bilmez.

## Runtime Start

    VM Starting
      -> NetworkPort.prepare_runtime
      -> NetworkRuntimePlan
      -> QEMU start
      -> process_id
      -> NetworkPort.bind_runtime_process
      -> VM Running

Her hata adiminda TurkuazVM'in sahip oldugu host network kaynagi cleanup edilir.

## Portable Managed NAT Strategy

`NetworkMode::ManagedNat` Windows ve Linux icin ayni hazirlik yolunu kullanir:

    ManagedNat
      -> PreparedNetworkBackend::UserNat
      -> QEMU -netdev user

Managed IPAM varsa QEMU argumanlari su sekilde genisler:

    net=<subnet>/<prefix>
    host=<gateway>
    dhcpstart=<vm-ip>
    hostfwd=<protocol>:<host-ip>:<host-port>-<guest-ip>:<guest-port>

Bu backend hostta TAP, bridge veya NAT interface olusturmaz.

## Windows Advanced Strategy

`private` ve explicit `bridge` profilleri Windows helper/TAP altyapisini kullanabilir. Bu yol varsayilan degildir.

- Existing TAP dogrulanir.
- Private fabric managed TAP + Network Bridge kullanabilir.
- Gereken host kaynaklari runtime lease ile sahiplenilir.
- OpenVPN `tapctl` yalniz Advanced TAP gerektiren senaryolarda kullanilir.

## Linux Advanced Strategy

Linux bridge attachment icin uc yol vardir:

1. Explicit existing TAP.
2. Ayarlanmis QEMU bridge helper.
3. `allow_managed_tap` aciksa root yetkili managed TAP fallback.

Varsayilan `managed_nat` bunlarin hicbirini gerektirmez.

## Runtime Lease

Her prepare isleminde `data/runtime/network/<vm-id>.yml` lease yazilir. Lease su bilgileri tasir:

- VM id
- network id
- backend turu
- TAP/bridge adi varsa adi
- kaynagin TurkuazVM sahipligi
- QEMU process id

QEMU baslamadan once PID `null` olur. Basarili start sonrasinda process id lease'e baglanir.

## Crash Recovery

Engine yeniden basladiginda lease dosyalari taranir.

- PID hala yasiyorsa lease aktif kabul edilir.
- User NAT lease host kaynagi tasimadigi icin dosya cleanup yeterlidir.
- Owned Advanced managed TAP kaynagi varsa helper/interface cleanup denenir.
- Existing TAP ve host bridge TurkuazVM tarafindan silinmez.
- v0.31.x managed TAP lease'i portable-only VM icin yeni User NAT startini bloklamaz.

## QEMU Mapping

Prepared backend mapping:

    UserNat
      -> -netdev user

    HostTap
      -> -netdev tap,ifname=...,script=no,downscript=no

    HostBridge
      -> -netdev bridge,br=...,helper=...

QEMU syntax sadece `crates/qemu` icinde bulunur.
