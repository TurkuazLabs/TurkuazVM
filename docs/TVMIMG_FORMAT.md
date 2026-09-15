# 📄 Dosya Yolu: /turkuazvm/docs/TVMIMG_FORMAT.md
# 📌 Amac: TurkuazVM portable image paket formatini ve uyumluluk kurallarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: .tvmimg container yapisi, standart disk payloadlari, private-copy import policy ve export uyumlulugunu tanimlar
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# TurkuazVM Portable Image Format (.tvmimg)

## Hedef

`.tvmimg`, TurkuazVM'e ait portable image paket uzantisidir. Disk veri formati proprietary degildir. Paket icindeki disk ve Android artifactlari standart formatlarda tutulur.

Temel hedefler:

- TurkuazVM'e ait tek dosyalik image paketi.
- QEMU/KVM/virt-manager gibi araclarla uyumlu standart QCOW2/RAW payload.
- Android icin standart `boot.img`, `vendor_boot.img`, `super.img`, `userdata.img`, `vbmeta.img` ve ilgili artifactlar.
- Paket acildiginda veya export edildiginde vendor lock-in olmamasi.
- Runtime VM disklerinin varsayilan olarak birbirinden tamamen bagimsiz olmasi.

## Container

Schema 1 icin container tipi `ZIP64` olarak sabitlenmistir.

Uzanti:

`*.tvmimg`

ZIP64 secim nedenleri:

- 4 GiB uzeri dosyalari destekler.
- Standart arsiv araclariyla acilabilir.
- Manifest ve birden fazla artifact tek dosyada tasinabilir.
- QCOW2 gibi zaten optimize edilmis payloadlar `store` modu ile tekrar sikistirilmadan eklenebilir.

## Paket Yapisi

Ornek:

```text
my-android-image.tvmimg
  manifest.yml
  checksums.sha256
  disks/
    system.qcow2
  android/
    boot.img
    vendor_boot.img
    super.img
    userdata.img
    vbmeta.img
    composite.img
    bootloader.qemu
  metadata/
    provenance.yml
```

`manifest.yml` TurkuazVM'e ozel metadata tasir. Payload dosyalari standart formatta kalir.

## Runtime Izolasyon Policy

Varsayilan policy:

`private_copy_default: true`

Bir `.tvmimg` bir VM'e import/assign edildiginde runtime diski paylasimli backing file olarak kullanilmaz.

- Generic/Linux/Windows diskleri VM'e ozel QCOW2 olarak olusturulur veya full-convert edilir.
- Android `composite.img` registry artifacti sadece kurulum kaynagidir.
- Android runtime icin `qemu-img convert -f raw -O qcow2` ile VM'e ozel `os-private.qcow2` olusturulur.
- Iki VM ayni `.tvmimg` paketinden uretilse bile runtime diskleri birbirinden bagimsizdir.
- Linked clone/paylasimli backing ancak kullanici acikca sectiginde kullanilabilir.

## Uyumluluk

TurkuazVM import hedefleri ilerleyen fazlarda su standartlari destekleyecektir:

- `.qcow2`
- `.raw` / `.img`
- `.vmdk` -> qemu-img convert
- `.vdi` -> qemu-img convert
- `.tvmimg`

TurkuazVM export hedefleri:

- `.tvmimg` portable paket
- `.qcow2` standart QEMU disk
- `.raw` standart raw disk

## Guvenlik

- Paket entry path traversal reddedilir.
- Symlink payload varsayilan olarak reddedilir.
- Her artifact SHA-256 ile manifest/checksum listesinde dogrulanir.
- Import staging alaninda yapilir; validation bitmeden runtime registry'ye alinmaz.
- READY paketler immutable kabul edilir.
