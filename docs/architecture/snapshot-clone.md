# 📄 Dosya Yolu: /turkuazvm/docs/architecture/snapshot-clone.md
# 📌 Amac: TurkuazVM snapshot ve VM clone mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Offline QCOW2 snapshot, full clone, linked clone, backing-chain ve rollback sinirlarini belgeler
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Snapshot ve Clone Architecture

## Dependency Akisi

```text
Desktop View
    -> Desktop Controller
    -> DesktopService
    -> Engine API v2
    -> EngineApplicationService
        -> SnapshotService / CloneService
        -> VmRepositoryPort
        -> SnapshotPort / CloneStoragePort
        <- EngineStorageTool
        <- QemuImgTool
        <- qemu-img
```

Core katmani `qemu-img`, filesystem command, JSON veya Tauri bilmez.

## Snapshot Modeli

Snapshotlar yalniz `Stopped` VM durumunda olusturulur, restore edilir ve silinir. v0.3.0 snapshot backend'i QEMU internal QCOW2 snapshot mekanizmasidir. Bir VM'de birden fazla disk varsa ayni `SnapshotId` tum QCOW2 disklere uygulanir.

Create transaction:

```text
Validate VM + QCOW2 disks
    -> disk-1 snapshot create
    -> disk-2 snapshot create
    -> ...
    -> SnapshotRecord aggregate'e ekle
    -> machine.yml save
```

Disk create adimlarindan biri hata verirse daha once olusturulan snapshotlar ters sirada silinmeye calisilir. Repository save hata verirse fiziksel snapshotlar rollback edilir.

Restore ve delete islemleri multi-disk qemu-img operasyonlari oldugu icin storage katmaninda tek host-level atomic primitive yoktur. Bu nedenle service `PartialRestore` ve `PartialDelete` typed errorlari ile hangi disklerin tamamlandigini raporlayabilir.

## Snapshot Persistence

`machine.yml` schema version 5 ile snapshot metadata eklenir:

```text
snapshots:
  records:
    - id: before-update
      name: Before Update
      created_at_unix_ms: 1787080000000
      disk_ids:
        - system
```

Schema 1-4 manifestleri `snapshots.records = []` varsayimi ile okunur.

## Full Clone

Full clone her source diski `qemu-img convert` ile hedef VM klasorune bagimsiz olarak kopyalar. Guest `media/` ve `firmware/` agaclari da hedefe kopyalanir.

Hedef VM:

- kaynak CPU/RAM ayarlarini alir,
- kaynak acceleration secimini alir,
- guest boot ayarlarini alir,
- disk attachmentlarini alir,
- snapshot gecmisini almaz,
- network adapterlerini almaz.

Network'in kopyalanmamasi duplicate MAC ve host port bind cakismalarini engeller.

## Linked Clone

Linked clone yalniz QCOW2 disklerde desteklenir. Kaynak VM'nin aktif diskini backing file yapmak guvenli degildir; kaynak disk daha sonra degisirse hedef zinciri bozulabilir. Bu nedenle v0.3.0 once hedef VM disk dizininde immutable local base kopyasi olusturur, sonra hedef aktif QCOW2 overlay bu base'e baglanir.

```text
source/system.qcow2
        |
        | qemu-img convert
        v
target/.system-clone-base.qcow2
        ^
        |
backing file
        |
target/system.qcow2
```

Kaynak VM clone sonrasinda yeniden calistirilabilir ve hedef chain bundan etkilenmez. Hedef overlay backing dosyasina mutlak path ile baglanir; bu nedenle v0.3.0 linked clone klasorunu baska konuma tasimadan once ileride eklenecek rebase/relocation use-case'i gereklidir.

## Clone Transaction Ownership

Clone baslamadan once `prepare_clone_target` hedef VM klasorunun mevcut olmadigini dogrular ve yeni root'u olusturur. Bu noktadan sonra hedef klasor bu transaction'a aittir. Disk veya asset copy hata verirse `cleanup_clone` yalniz bu transaction tarafindan sahiplenilen hedef agaci temizler.
