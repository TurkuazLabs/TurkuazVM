# 📄 Dosya Yolu: /turkuazvm/docs/ANDROID_IMAGE.md
# 📌 Amac: Turkuaz Android otomatik provisioning, AOSP build, register ve assignment kullanimini aciklar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Resmi Android CI auto-provisioning ve opsiyonel Linux AOSP custom image operasyonunu belgeler
# Bagimli Oldugu Katman: Service | Repo | Tool | View | Config

# Turkuaz Android Image

## 1. Otomatik Kurulum - Onerilen Ilk Test

Desktop -> Android Images ekraninda image kaydi tanimlanir ve `Otomatik Kur` secilir. Engine API istegi yalniz background install jobunu baslatir; uzun indirme request thread'ini bloke etmez.

Worker resmi Android CI latest Cuttlefish x86_64 build ID'sini kesfeder, ayni build'e ait device image ve `cvd-host_package.tar.gz` paketini indirir. v0.12.2 operasyon katmani kurulum iptali, retry, staging cleanup, hiz/ETA telemetry ve local install.log goruntulemesini ekler. Archive path preflight, SHA-256 provenance, bootloader discovery ve Rust sparse/GPT composite assembly tamamlaninca state `READY` olur. Hata durumunda `FAILED` ve hata mesaji registry'ye yazilir. Desktop `INSTALLING` durumunu periyodik yeniler. v0.12.2 ile kart uzerinde typed kurulum asamasi, indirilen byte, varsa toplam/yuzde ve install log yolu gosterilir.

Indirme baslamadan once `android.image.distribution.minimum_free_disk_gib` ile bos disk kontrol edilir. Device/host paketleri retry ile indirilir; ayni build ID tekrar denenirse partial dosya `curl --continue-at -` ile devam ettirilir. CDN resume istegini reddederse partial dosya silinir ve indirme sifirdan bir kez daha denenir. Build ID degisirse eski staging download temizlenir. Windows launcher `curl.exe` ve `tar.exe` araclarini bu akistan once dogrular.

Stok Cuttlefish image TurkuazInputAgent icermedigi icin `guest_agent_included=false` ve `persistent_multi_touch=false` capability ile kaydedilir. Ilk boot ADB + display readiness ile tamamlanabilir; custom agent'li AOSP yolu asagida korunur.

## 2. Custom AOSP Yolu
### AOSP Kaynagini Hazirla

Linux hostta `guest/android-image/scripts/prepare_aosp_source.sh` kullanilir. Config'teki `android.image.source_root` ayni source root'u gostermelidir.

### Image Kaydi Tanimla

Desktop -> Android Images ekraninda image ID ve ad girilir. Bu asama build yapmaz.

### Build Planini Al

Build Plan butonu Engine'in source root, output root, lunch target, build script ve minimum disk bilgisini dondurur.

### Linux Build Scriptini Calistir

`guest/android-image/scripts/build_turkuaz_android_image.sh` source root, output root ve image ID ile calistirilir.

Script:

- Turkuaz Cuttlefish product overlay'ini source tree'ye kopyalar.
- TurkuazInputAgent'i AOSP product paketine dahil eder.
- `aosp_current` + `userdebug` lunch targetini build eder.
- boot/super/userdata ve bulunan ek artifactleri bundle'a kopyalar.
- Cuttlefish U-Boot artifactini `bootloader.qemu` olarak kaydeder.
- A/B Android partitionlarini tek GPT `composite.img` icinde birlestirir.
- Android release, SDK ve source revision bilgilerini `build-info.yml` icine yazar.

### Register

Desktop'ta Register butonu bundle'i tarar. `boot.img`, `super.img`, `userdata.img`, `bootloader.qemu` ve `composite.img` ile checksum'lar dogrulaninca image `READY` olur.

## 3. Android VM'ye Ata

Android Runtime ekraninda READY image secilir ve VM Stopped durumdayken atanir. Ilk Start sirasinda Engine composite template icin VM ve image-ozel QCOW2 overlay ve pflash state uretir; ortak image bundle dogrudan yazilabilir disk olarak kullanilmaz.

## 4. First Boot

Engine Android VM ilk acilisinda ADB readiness bekler ve display profilini uygular. Atanan image capability `guest_agent_included=true` ise TurkuazInputAgent AccessibilityService mevcut servis listesini koruyarak etkinlestirilir ve Guest Agent Ping/Pong handshake beklenir. Stok otomatik Cuttlefish image bu adimi atlar. Basarili akista assignment `ready`, hata durumunda `failed` olur ve boot attempt sayaci saklanir.

## Sinirlar

- AOSP source code TurkuazVM ZIP'ine dahil degildir.
- Custom AOSP build Engine API request'i icinde calistirilmaz; otomatik resmi dagitim indirmesi Engine background workerinda calisir.
- ARM/ARM64 translation dahil degildir.
- Gercek AOSP/QEMU boot testi Linux build hostu ve uyumlu QEMU/Cuttlefish artifactleri gerektirir.
