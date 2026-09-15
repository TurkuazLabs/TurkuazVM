# 📄 Dosya Yolu: /turkuazvm/docs/architecture/android-image-foundation.md
# 📌 Amac: Turkuaz Android image build, registry, assignment ve artifact sinirlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: AOSP build ile resmi Android CI auto-provisioning yollarini ayirir ve Engine'e checksum dogrulamali image registry saglar
# Bagimli Oldugu Katman: Service | Repo | Tool | Config

# Android Image Foundation

## Hedef

TurkuazVM Engine AOSP kaynak agacini request thread'i icinde derlemez. Engine yalniz build planini uretir, artifact bundle'ini kaydeder ve READY image'i Android VM'ye atar.

## Dikey Akis

Desktop -> Engine API v17 -> AndroidImageApplicationService -> AndroidImageService -> AndroidImageRepositoryPort / AndroidImageBuilderPort / AndroidImageDistributionPort

Repository adapteri image metadata ve VM assignment bilgisini YAML olarak saklar. Builder adapteri Linux AOSP source tree ve build output bundle ile ilgilenir. Distribution adapteri resmi Android CI Cuttlefish buildlerini background worker uzerinden indirip READY bundle haline getirir.

## Otomatik Dagitim Akisi

1. `InstallAndroidImageDistribution` image state'ini `Installing` yapar ve Engine background worker baslatir.
2. Android CI branch `status.json` LKG kaydindan numeric build ID kesfedilir; concrete `BUILD_INFO` yalniz opsiyonel metadata icin okunur.
3. Ayni build device image ve `cvd-host_package.tar.gz` staging alanina indirilir; host paketi Windows runtime olarak calistirilmaz.
4. Archive path preflight ve SHA-256 provenance uygulanir.
5. Host tar agaci tamamen acilmaz; archive listing icinden yalniz x86_64 QEMU bootloader uyesi secilir ve stdout stream ile `bootloader.qemu` olarak bundle'a yazilir. Sparse partitionlar Rust tool ile GPT `composite.img` icine yazilir.
6. Bundle staging -> final swap ile aktive edilir ve registry schema 2 image'i `Ready` yapar.
7. Engine restartinda yarida kalmis `Installing` kayitlari `Failed` durumuna toparlanarak yeniden denemeye acilir.

## Build Akisi

1. `DefineAndroidImage` image aggregate kaydini olusturur.
2. `PrepareAndroidImageBuild` Linux source/output/script bilgisini iceren plan uretir ve state'i `BuildPlanned` yapar.
3. `prepare_aosp_source.sh` resmi AOSP source checkout'unu hazirlar.
4. `build_turkuaz_android_image.sh` custom Cuttlefish product ve TurkuazInputAgent'i source tree'ye uygular, AOSP build'i calistirir ve bundle olusturur.
5. Build script Cuttlefish partitionlarini GPT `composite.img` icinde birlestirir ve `bootloader.qemu` artifactini bundle'a alir.
6. `RegisterAndroidImageBuild` bundle artifactlerini SHA-256 ile tarar ve gerekli boot artifactleri varsa image'i `Ready` yapar.
7. `AssignAndroidImage` yalniz Stopped + Android guest profile VM'ye READY image atar.
8. Ilk Start, composite template icin VM ve image-ozel QCOW2 overlay/pflash hazirlar ve typed runtime media planini QEMU adapterine verir.
9. ADB/display/Guest Agent readiness tamamlaninca assignment `Ready`; hata durumunda `Failed` olur.

## Registry

Image metadata:

`data/android-images/<image-id>/image.yml`

VM assignment:

`data/machines/<vm-id>/android-image.yml`

Build output:

`data/android-image-builds/<image-id>/`

## Guvenlik ve Butunluk

- Image ID path traversal kabul etmez.
- Artifact pathleri registry tarafinda relative tutulur.
- Artifactler kayit sirasinda SHA-256 ile fingerprint edilir.
- READY state icin boot, super, userdata, bootloader ve composite disk artifactleri zorunludur.
- VM assignment yalniz Android guest profile ve Stopped state icin yapilir.
- AOSP build hostu Linux olarak sinirlanir.
- Resmi Cuttlefish host runtime Linux/KVM odaklidir; Windows-native TurkuazVM akisi `launch_cvd` ve diger Linux host binarylerini calistirmaz.
- `cvd-host_package.tar.gz` Windows tarafinda tam extract edilmez; yalniz x86_64 QEMU bootloader uyesi kontrollu olarak stream edilir.
- ARM translation v0.10.0 image capability'sinde false kalir.

## v0.12.0 Boot Siniri

Composite bundle registry tarafinda immutable template olarak kalir. Runtime yazmalari VM ve image-ozel QCOW2 overlay'e gider. ARM/ARM64 translation ve production-ready Windows accelerated Android renderer bu kilometre tasinin kapsaminda degildir.

## v0.12.2 Provisioning Operations

Distribution Port artik prepare/install/progress/cancel/cleanup contractlarini tasir. Cancel marker Tool katmaninda kalici tutulur; Service state kurallarini uygular. FAILED staging temizligi Service tarafindan READY/INSTALLING invariantlari ile korunur. Desktop log viewer yalniz local data-root altindaki `install.log` dosyasini acar.
