# 📄 Dosya Yolu: /turkuazvm/docs/architecture/network-foundation.md
# 📌 Amac: TurkuazVM portable QEMU NAT, private fabric ve Advanced Bridge/TAP network mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.32.0
# Aciklama: IPAM, QEMU User NAT, hostfwd, Advanced TAP/Bridge ve runtime cleanup sinirlarini belgeler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# Network Foundation

## Dikey Akis

    Desktop View
        -> DesktopController
        -> DesktopService
        -> Engine API
        -> EngineApplicationService
        -> NetworkService
        -> VmRepositoryPort + NetworkPort
        -> NativeNetworkTool
        -> QemuCommandBuilder

Controller yalniz request tasir. Profil secimi, IP tahsisi ve servis yayinlama kurallari Service katmanindadir. Host komutlari Tool katmaninda kalir.

## Turkuaz NAT

Varsayilan profil `managed_nat` olur ve Windows/Linux hostlarda QEMU User NAT kullanir.

- Fabric ID: `turkuaz-net-01`.
- Varsayilan subnet: `192.168.240.0/24`.
- Gateway: `192.168.240.1`.
- IPAM havuzu: `.10-.199`.
- Her VM attachment kalici MAC, IPv4, prefix, gateway ve `fabric_id` bilgisini manifestte saklar.
- `NativeNetworkTool` bu profili `PreparedNetworkBackend::UserNat` olarak hazirlar.
- `QemuCommandBuilder` IPAM bilgisini `net`, `host` ve `dhcpstart` parametrelerine cevirir.

Portable runtime zinciri:

    VM virtio-net/e1000
        -> QEMU -netdev user
        -> QEMU SLIRP NAT
        -> Host Internet

Bu yol Windows TAP, Windows Network Bridge, WinNAT veya OpenVPN GUI gerektirmez.

## Guest IP ve Host Erisimi

Guest VM TurkuazVM IPAM tarafindan ayrilan private IPv4 adresini QEMU DHCP uzerinden alir. Bu adres QEMU User NAT icindedir ve hosttan dogrudan route edilmez.

Hosttan guest servisine erisim servis yayinlama ile yapilir:

    127.0.0.1:2201 -> guest:22
    127.0.0.1:8081 -> guest:80
    127.0.0.1:8441 -> guest:443

Varsayilan publish bind adresi `127.0.0.1` olur. Boylece servis acikca istenmeden fiziksel LAN'a acilmaz.

## Private Network

`private` profili gelismis host fabric senaryosudur. Windows tarafinda managed TAP/bridge ve DHCP/IPAM zincirini kullanir ancak Internet NAT olusturmaz. Bu profil varsayilan degildir ve platform-specific host network kaynagi gerektirebilir.

## Bridge / Existing TAP

`bridge` profili ileri seviye fiziksel LAN entegrasyonu icindir.

- Windows: mevcut TAP adapteri acikca secilir.
- Linux: mevcut TAP veya host bridge hedefi kullanilir.
- Linux managed TAP yalniz deployment ayari aciksa kullanilir.
- QEMU command syntax core domain'e sizmaz.

OpenVPN/TAP yalniz bu gelismis senaryolar icin opsiyonel dependency olarak kalir.

## QEMU NAT Uyumluluk Profili

`user_nat` geriye donuk uyumluluk profilidir. QEMU User NAT kullanir ancak TurkuazVM managed IPAM zorunlulugu yoktur. Yeni VM'lerde varsayilan profil `managed_nat` olur.

## Servis Yayinlama

`managed_nat` ve `user_nat` attachmentlari TCP/UDP servis yayinlama kurali tasiyabilir.

- Kurallar QEMU `hostfwd` argumanina cevrilir.
- Varsayilan host bind `127.0.0.1` olur.
- Private ve Bridge profillerinde port forward domain seviyesinde reddedilir.
- Ayni protocol + host bind port cakismasi domain seviyesinde reddedilir.

## Runtime Lease ve v0.31.x Gecisi

Her VM runtime hazirliginda `data/runtime/network/<vm-id>.yml` lease tutulur.

v0.31.x managed NAT lease'i `ManagedTap` kaydi tasiyabilir. v0.32.0 portable NAT'a gecen bir VM icin eski TAP cleanup basarisiz olsa bile stale lease yeni QEMU User NAT baslangicini bloklamaz; lease kaydi kontrollu sekilde kaldirilir. Advanced Private/Bridge kaynaklarinda fail-closed cleanup davranisi korunur.

## Persistence

`machine.yml` manifest schema `6` network attachment icin su alanlari saklar:

- attachment ID
- `fabric_id`
- mode
- device model
- MAC
- managed IPv4/prefix/gateway/DNS
- servis yayinlama kurallari
- bridge hedefi

## Guvenlik ve Sahiplik

- VM network mutation yalniz `Stopped` durumda yapilir.
- Managed adres private IPv4 olmak zorundadir.
- Multicast, broadcast ve sifir MAC reddedilir.
- QEMU NAT host publish varsayilani loopback'tir.
- TurkuazVM yalniz kendisinin olusturdugu managed TAP kaynagini siler.
- Advanced Windows helper Administrator yetkisi gerektirebilir.
- Varsayilan Turkuaz NAT icin OpenVPN veya TAP runtime probe zorunlu degildir.
