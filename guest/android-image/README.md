# 📄 Dosya Yolu: /turkuazvm/guest/android-image/README.md
# 📌 Amac: Turkuaz Android Image build, registry ve boot media akisinin operator kullanimini aciklar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: AOSP kaynak hazirligi, custom product overlay, composite bundle, registration ve ilk boot adimlarini belgeler
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Turkuaz Android Image Boot Integration

Bu klasor AOSP kaynak kodunu TurkuazVM repository'sine kopyalamaz. AOSP ayri bir Linux build hostunda tutulur.

## Source hazirlama

`prepare_aosp_source.sh` resmi Repo init/sync modelini kullanir ve build hostunda 400 GiB minimum free disk kontrolu yapar.

## Build modeli

1. `scripts/prepare_aosp_source.sh` ile AOSP `android-latest-release` source tree hazirlanir.
2. Engine `PrepareAndroidImageBuild` ile build planini uretir.
3. `scripts/build_turkuaz_android_image.sh` custom product ve TurkuazInputAgent kaynaklarini AOSP tree altina uygular.
4. `turkuazvm_cf_x86_64_phone-aosp_current-userdebug` build edilir.
5. Boot, dynamic partition, bootloader ve persistent partition artifactleri image-id bazli bundle klasorune kopyalanir.
6. `assemble_turkuaz_android_composite.py` U-Boot environment ve A/B GPT partition tablosu ile `composite.img` uretir.
7. Engine `RegisterAndroidImageBuild` artifact size, SHA-256 ve metadata kaydini olusturur.
8. VM ilk Start sirasinda composite template icin VM ve image-ozel QCOW2 overlay/pflash state hazirlanir.
9. ADB readiness, display provisioning ve TurkuazInputAgent Ping/Pong tamamlaninca assignment `Ready` olur.

## v0.11.8 siniri

Bu release Android image build/registry foundation ile QEMU boot media entegrasyonunu birlestirir. ARM/ARM64 native translation ve production-ready Windows accelerated Android gaming renderer dahil degildir; bunlar v0.13.0 Android Gaming Renderer kilometre tasina ayrilmistir.
