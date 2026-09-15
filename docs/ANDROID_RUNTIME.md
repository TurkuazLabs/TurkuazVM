# 📄 Dosya Yolu: /turkuazvm/docs/ANDROID_RUNTIME.md
# 📌 Amac: TurkuazVM Android Runtime Foundation kullanim akisini ve sinirlarini aciklar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Android guest profile, ADB readiness, package lifecycle, display profile ve input foundation kullanim rehberidir
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# Android Runtime ve Boot Integration

## Kapsam

v0.11.8, v0.6.0 ile kurulan Android control-plane'ini v0.10.0 image registry ile birlestirir ve atanmis Turkuaz Android image'ini QEMU boot zincirine baglar.

Boot ozellikleri:

- Registered image `composite.img` template diskini ve `bootloader.qemu` firmware artifactini tasir.
- Engine her VM ve image icin backing'siz, bagimsiz `os-private.qcow2` ve pflash state uretir.
- QEMU Android runtime media planini generic VM disklerinden ayri typed contract ile alir.
- Guest tarafinda ADB TCP servisi configure edilen guest portunda erisilebilir olur.
- Varsayilan guest ADB portu `5555/TCP` olarak tanimlidir.

## VM Olusturma

Desktop `Yeni VM` ekraninda guest profile:

```text
Android Gaming
```

secilir. Bu secim VM aggregate icinde `GuestProfile::Android` olarak saklanir.

## Runtime Configure

VM `Stopped` durumundayken `Android` panelinden Configure calistirilir.

Engine:

1. VM profile kontrolu yapar.
2. Mevcut `android.yml` varsa ADB host portunu korur.
3. Dedicated `android-nat` attachment varsa profile ile uyumunu dogrular.
4. Yoksa 5600-5699 araligindan bos loopback port ayirir.
5. Host portunu guest ADB portuna TCP forward eder.
6. Android profile'i `android.yml` olarak kaydeder.

Provisioning yari yolda hata verirse o islemde yeni eklenen network attachment compensation ile geri alinir.

## Device Readiness

VM `Running` durumundayken:

- Status: ADB state, boot-completed, SDK, ABI ve model bilgisini sorgular.
- Wait Ready: configured timeout boyunca device'in hazir olmasini bekler.
- Apply Display: width/height ve density override'larini uygular.
- First boot: Guest Agent AccessibilityService provision edilir ve Ping/Pong readiness beklenir.
- Basari: Android image assignment `ready` olur.
- Hata: assignment `failed` olur, boot attempt sayaci korunur ve VM kontrollu durdurulur.

## Package Lifecycle

APK dosyalari `android.package_root` altinda bulunur. Desktop yalniz relative `.apk` yolu yollar.

Desteklenen islemler:

- APK install/replace
- Package list
- Package uninstall
- Launcher activity resolve + launch
- Force stop

Path traversal, absolute path ve package-root disina symlink escape reddedilir.

## Input Foundation

Typed input actionlari:

- Tap
- Swipe
- KeyEvent
- Text

v0.6.0 bunlari ADB shell input adapterine yollar. Bu yol functional test ve control-plane icindir; gaming icin nihai low-latency input pipeline degildir.

## Persistence

Android profile:

```text
data/machines/<vm-id>/android.yml
```

Normal VM manifestinden ayridir:

```text
data/machines/<vm-id>/machine.yml
```

Bu ayrim Android package/input/runtime detaylarinin virtualization domain'ine sizmasini engeller.

## Guvenlik

- ADB host endpoint loopback olmak zorundadir.
- Host ADB portu typed range icinden ayrilir.
- Remote client arbitrary APK host path gonderemez.
- APK canonical path package root icinde kalmak zorundadir.
- QMP remote client'a acilmaz.

## Bilincli Olarak Bu Surumde Yok

- Google Play entegrasyonu
- ARM/ARM64 native ABI translation
- Windows icin production-ready accelerated Android gaming renderer
- Remote low-latency gaming display/input streaming
- Anti-cheat veya emulator identity bypass
