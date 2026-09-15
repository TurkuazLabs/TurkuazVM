# 📄 Dosya Yolu: /turkuazvm/docs/architecture/storage-foundation.md
# 📌 Amac: TurkuazVM storage domain, port, qemu-img ve manifest persistence mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Disk mutation ve VM persistence sorumluluklarini katmanlara ayirir
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool

# Storage Foundation

## Akis

    EngineApiController
          |
          v
 EngineApplicationService
          |
          v
    StorageService
       /       \
      v         v
VmRepository  StoragePort
      |         |
      v         v
YamlVmRepo   QemuImgTool
      |         |
      v         v
machine.yml  qemu-img


## v0.12.4 Host Storage Foundation

- `HostStorageTool` filesystem kapasitesini `fs2` ile okur; PowerShell veya `df` stdout parse edilmez.
- `StorageHostService` data root total/free byte, qemu-img readiness ve varsayilan disk policy bilgisini tek raporda uretir.
- Desktop `Storage` ekrani bu raporu Engine API v14 uzerinden gosterir.
- Yeni disk create islemi `EngineApiController -> EngineApplicationService -> StorageService -> StoragePort -> QemuImgTool` yonunu korur.
- Varsayilan runtime disk `qcow2`, bus `virtio`, boyut ve portable image policy config'ten gelir.

## Portable Image Policy

TurkuazVM portable paket uzantisi `.tvmimg` olarak sabitlenmistir. `.tvmimg` proprietary disk formati degildir; ZIP64 container icinde manifest/checksum ve standart payload tasir. Disk payload'i QCOW2/RAW, Android payload'i boot/super/vendor_boot benzeri standart Android artifact'lari olabilir.

Varsayilan `private_copy_default: true` politikasinda portable paket veya Android registry bundle yalniz kurulum kaynagidir. Calisan VM ortak writable/backing disk kullanmaz. Android icin registry `composite.img` dosyasindan `qemu-img convert -f raw -O qcow2` ile VM ve image-ozel `os-private.qcow2` uretilir. Linked clone ancak kullanici tarafindan acikca istenen ayri bir clone use-case'idir.

Ayrintili container contract'i `docs/TVMIMG_FORMAT.md` dosyasindadir.

## Domain Kurallari

- DiskId typed value object'tir.
- Disk path absolute olamaz.
- Disk path parent directory ile VM klasorunden cikamaz.
- Virtual size sifir olamaz.
- Ayni DiskId bir VM'ye iki kez attach edilemez.
- Offline qemu-img mutation sadece Stopped VM icin calisir.
- v0.1.3 shrink desteklemez.

## Runtime Layout

    data/machines/<vm-id>/machine.yml
    data/machines/<vm-id>/disks/<disk-id>.qcow2

Manifest disk yolunu relative tutar. QemuImgTool ve QemuCommandBuilder ayni data_root uzerinden absolute host yolunu runtime aninda cozer.

## Persistence

YamlVmRepository core domain tiplerini serde ile annotate etmez. Repository kendi DTO modellerini kullanir ve domain <-> manifest conversion yapar. Boylece core crate YAML veya serde bilmez.

## Compensation

Disk create basarili olup VM manifest save basarisiz olursa olusturulan disk silinmeye calisilir.

Resize storage mutation fiziksel olarak gerceklestikten sonra manifest save basarisiz olursa eski boyuta geri donulmez. Shrink veri kaybi riski tasidigi icin service RepositoryDivergedAfterStorageMutation hatasi dondurur. Bir sonraki repair/reconcile milestone'unda fiziksel image bilgisi manifest ile tekrar eslestirilecektir.
