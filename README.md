# 📄 Dosya Yolu: /turkuazvm/README.md
# 📌 Amac: TurkuazVM v0.41.2 FULL kaynak agacini ve Baglanti Merkezi calisan-VM erisim yamasini aciklar
# 📌 Modul - Markdown
# Version: 0.41.2
# Aciklama: Calisan Turkuaz NAT VM icin SSH/RDP host yayinini onayli otomatik stop/restart ile hazirlar; v0.41.1 workspace ve v0.40.14 runtime duzeltmelerini korur
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# TurkuazVM v0.41.2


## v0.41.2 Baglanti Merkezi Calisan-VM Erisim Yamasi

- Calisan Turkuaz NAT VM icin `SSH Erisimini Hazirla` ve `RDP Erisimini Hazirla` artik kilitli degildir.
- Hazirlama aksiyonu calisan VM icin acik onay ister; VM kisa sure durdurulur, localhost port yayini eklenir ve VM yeniden baslatilir.
- Stop/publish/restart is kurali Desktop Service katmanindadir; Controller logic eklenmemistir.
- Publish basarisiz olursa Service VM'yi onceki calisma durumuna geri dondurmeyi dener ve recovery sonucunu hata mesajina ekler.
- Port yayini olustuktan sonra Baglanti Merkezi guncel ag durumunu tekrar yukler; Test Et ve Baglan aksiyonlari hedef hazir oldugunda aktif olur.
- SSH/RDP guest servisinin guest isletim sistemi icinde kurulu ve aktif olmasi yine gereklidir; port yayini tek basina guest servisi kurmaz.
- v0.41.2 regression contract eski `yalniz VM dururken` dead-end davranisinin geri gelmesini engeller.


## v0.41.1 Workspace Kullanilabilirlik Yamasi

- 1920x1080 ekranda kucuk kalan VM kontrol hedefleri, VM listesi, sekmeler ve kaynak bilgileri buyutuldu.
- Hic VM yokken sol liste ve sag detayda iki ayri bos mesaj gostermek yerine tek onboarding alani kullanilir.
- Bos-filo ekraninda Yeni VM Olustur ve Goruntu Merkezi ana aksiyonlari ile 3 adimli ilk kullanim akisi gorunur.
- Ana Sayfaya Yeni VM, Goruntu Merkezi ve VM Kontrol Merkezi hizli aksiyon kartlari eklendi.
- Goruntu Merkezi standart ISO karti 680px dar merkez kutusu yerine genis iki kolonlu workspace duzeni kullanir.
- v0.41.0 kontrol merkezi davranisi ve v0.40.14 Android runtime wiring korunur.
- v0.41.1 UI regression contract ve FULL static recovery gate ile korunur.

## v0.41.0 VM Kontrol Merkezi

- VM detay ekrani buyuk onizleme karti yerine kontrol-oncelikli komut merkezi kullanir.
- Baslat/Konsol, Durdur ve Baglan birincil kontrolleri secili VM ust alaninda her zaman gorunur.
- Donanim, Disk, Ag ve Snapshot hizli kontrolleri ayni komut alaninda toplanir.
- CPU, bellek, disk ve IPv4 bilgileri kompakt kaynak seridinde gorunur.
- Genel sekmesi Hizli Islemler ve Calisma Hazirligi panelleriyle gunluk yonetimi tek ekrana toplar.
- Android image eksigi dogrudan duzeltme aksiyonu verir.
- VM silme islemi birincil kontrol satirindan ayrilarak daha guvenli bir yonetim alanina tasinmistir.
- Sol VM listesi durum noktasina ek olarak okunabilir Calisiyor/Kapali/Sorunlu etiketi gosterir.
- v0.41.0 UI regression contract ve FULL static recovery gate PASS durumundadir.
- Rust/Cargo derlemesi bu paketleme ortaminda dogrulanmamistir; Windows launcher uzerinde tekrar test edilmelidir.


## v0.40.14 Android Emulator Port Config Wiring Hotfix

- Windows Cargo buildini `-D dead-code` altinda durduran `AndroidSdkEngineConfig.emulator_console_port_min/max` alanlari artik runtime media Tool tarafinda gercekten kullanilir.
- Android SDK console port araligi `config/download-sources.yml` -> Engine Config -> Android Image Service -> Android Runtime Media Tool zinciriyle tasinir.
- Android Application Service icindeki sabit `5555..=5681` ADB araligi kaldirildi; `config/turkuazvm.yml` ADB araligi kullanilir.
- Runtime media katmanindaki ayni magic ADB araligi kaldirildi; ADB portu console port ciftine cevrilip SDK config araliginda dogrulanir.
- Workspace `dead_code = "deny"` politikasi korunur; lint bastirma eklenmedi.
- v0.40.14 regression ve launcher structure gate, config alanlari tekrar tanimlanip tuketilmez hale gelirse fail-closed durur.
- v0.40.13 Android 10-17 resmi SDK System Image provider ve WHPX runtime mimarisi aynen korunur.

## v0.40.13 Windows Android SDK System Image Provider

- Windows Android 10-17 ana provideri artik `android_sdk`; historical `ci.android.com` target tahmini ana indirme yolu degildir.
- Release/API eslemesi 10/29, 11/30, 12/31, 12L/32, 13/33, 14/34, 15/35, 16/36 ve 17/37 olarak configte tutulur.
- System Image ve Android Emulator archive adresleri resmi Google repository XML kataloglarindan dinamik cozulur.
- AOSP `default` variant once denenir; `google_apis` ve `google_apis_playstore` fallback olarak kalir.
- Android image manifest schema v4 ile `sdk_emulator` runtime kind ve Kernel/Ramdisk/System/Userdata artifact rolleri persist edilir.
- VM baslatmada VM-ozel AVD hazirlanir ve Android Emulator `-accel auto` ile WHPX destekli calisir.
- Android Emulator ilk v0.40.13 asamasinda kendi resmi penceresinde acilir; embedded TurkuazDisplay sonraki performans fazidir.
- Legacy Cuttlefish/Android CI provider kodu fallback ve tarihsel uyumluluk icin korunur.
- Yeni VM sihirbazinda Android install `Logu Ac` ve `Iptal` handlerlari tamamlanmistir.
- Engine API v24 korunur.

## v0.40.12 Windows PowerShell 5.1 Process Collection Hotfix

- `Get-TurkuazNativeProcessesByExecutablePath` generic `List[object]` yerine plain PowerShell array kullanir.
- Windows PowerShell 5.1 tarafinda gorulen `Bagimsiz degisken turleri eslesmiyor` binder hatasi kapatilir.
- Exact executable-path eslestirme, stale Engine/Display cleanup ve Desktop already-running davranisi korunur.
- v0.40.10 Engine StartVm crash diagnostics ve Android resolver degisiklikleri korunur.

## v0.40.11 Windows Runtime Binary Lock Hotfix

- Ayni workspace icin `turkuazvm-desktop.exe` zaten calisiyorsa launcher tekrar Cargo build baslatmaz.
- Desktop kapaliysa stale `turkuazvm-engine.exe` ve `turkuazvm-display.exe` processleri exact executable path ile temizlenir.
- Build oncesi runtime binary file locklari kontrollu olarak serbest birakilana kadar beklenir.
- Baska TurkuazVM surumlerinin veya baska Cargo projelerinin processleri sonlandirilmaz.
- v0.40.10 Engine StartVm crash diagnostics davranisi korunur.


## v0.40.10 Engine StartVm Crash Diagnostics Hotfix

- Android image attach basarili oldugu halde `StartVm` requestinde Desktop tarafinda gorulen generic Windows `os error 10054` semptomu icin Engine request boundary panic containment eklendi.
- Rust request panicleri artik Engine processini sessizce dusurmek yerine `engine_request_panic` kodlu structured Engine API hatasina cevrilir ve panic detayi Engine loguna yazilir.
- Engine gercekten process olarak sonlanirsa Desktop ayni generic transport mesajini gostermek yerine PID, exit status, Engine log yolu ve diagnostik tail bilgisini toplar.
- Mutating `StartVm` istegi guvenlik nedeniyle otomatik tekrar edilmez.
- v0.40.9 numeric build-id compile hotfixi ve v0.40.5 Android historical target resolver davranisi korunur.

## v0.40.9 Android CI Numeric Build-ID Compile Hotfix

- Windows Cargo buildinde `cannot find function numeric_build_id in this scope` ile duran `turkuazvm-guest` compile hatasi kapatildi.
- `collect_status_targets` tarafindan gercek production akisinda kullanilan `numeric_build_id` helperi geri getirildi; bu helper dead-code degildir ve `dead_code = "deny"` politikasini gevsetmez.
- Helper yalniz `BUILD_ID_FIELDS` alanlarini, JSON string/number degerlerini ve tamamen rakamsal build ID degerlerini kabul eder.
- Launcher structure gate artik production parserin kullandigi `numeric_build_id` sembolunu da zorunlu tutar.
- Eski v0.37.9 compatibility testi silinmis duplicate parser helperlarini istemek yerine mevcut production parseri dogrular.
- Yeni v0.40.9 regression testi unresolved helper regresyonunu fail-closed kilitler.
- v0.40.8 verifier, v0.40.7 dead-code cleanup ve v0.40.5 Android historical dynamic target discovery korunur.

## v0.40.8 Android CI Launcher Verifier Contract Hotfix

- v0.40.7'de production resolverdan kaldirilan `parse_last_known_good_build` sembolunu launcher `verify_structure.ps1` ve historical v0.40.0 regression testi hala zorunlu tutuyordu.
- Launcher structure gate artik `parse_status_target_candidates`, `parse_status_targets` ve `collect_status_targets` production parser yolunu dogrular.
- Historical source-resolver regression ayni production parser kontratina tasindi.
- Yeni v0.40.8 regression testi stale verifier/test sembollerinin tekrar geri donmesini fail-closed engeller.
- FULL recovery gate artik v0.40.0 Android source-resolver compatibility kontratini da her release icin calistirir.
- v0.40.7 dead-code temizligi ve v0.40.5 Android historical target discovery davranisi korunur.

## v0.40.7 Android CI Dead-Code Compile Hotfix

- `turkuazvm-guest` normal library buildinde `-D dead-code` nedeniyle derlemeyi durduran test-only legacy Android CI parser helperlari kaldirildi.
- Modern ve legacy `status.json` sekilleri artik ayni production `parse_status_target_candidates` parseri ile test edilir.
- `NESTED_BUILD_FIELDS`, `parse_last_known_good_build`, `parse_target_collection`, `target_entry_matches`, `parse_build_from_target_entry` ve `find_legacy_target_build` duplicate yolu kaldirildi.
- Workspace `dead_code = "deny"` politikasi korunur; hata warninge cevrilmedi ve lint gevsetilmedi.
- v0.40.6 release metadata, v0.40.5 Android dynamic target discovery, v0.40.4 softbuffer, v0.40.3 ISO lifecycle ve v0.40.2 display davranislari korunur.


## v0.40.6 Release Metadata Contract Hotfix

- Windows launcher'da preflight sonrasi `RELEASE_ENGINE_API_VERSION_NOT_FOUND` ile acilisin kesilmesine neden olan eksik release compatibility metadata duzeltildi.
- `RELEASE_STATUS` tekrar `engine_api_version`, `guest_agent_protocol_version` ve aktif schema surumlerini authoritative compatibility bolumunde tasir.
- Python FULL recovery gate artik release status metadata ile Rust Engine API ve Android guest-agent protocol kaynaklarini capraz dogrular.
- Yeni regression testi ayni metadata'nin bir sonraki paketlemede tekrar dusmesini engeller.
- v0.40.5 Android historical CI resolver degisiklikleri aynen korunur.

## v0.40.5 Android Historical CI Target Discovery Hotfix

- Android 11-16 ve 12L image resolver exact target adina bagli degildir.
- Android CI status katalogundaki historical x86_64 Cuttlefish phone userdebug targetlari runtime'da kesfedilir.
- Exact configured target her zaman onceliklidir; fallback target ancak BUILD_INFO SDK ve same-build artifact kontrollerini gecerse kullanilir.
- Android 12L icin ek historical branch fallbackleri vardir.
- Resolver basarisiz olursa log status icindeki gorulen targetlardan diagnostik ozet verir.

## v0.40.4 Launcher Softbuffer Contract Hotfix

- Windows preflight sonrasi gorulen `SOFTBUFFER_DISPLAY_CONTRACT_MISSING: buffer[host_row + x]` launch blokaji kapatildi.
- Sorun renderer degil, `verify_structure.ps1` icinde v0.40.1 piksel dongusunden kalmis stale exact-source kontratiydi.
- v0.40.2 renderer `host_x` ve cached scale-map kullandigi halde verifier eski `x` degiskenini ve kaldirilmis `0x00FF_FFFF` maskesini ariyordu.
- Structure gate artik yerel degisken adina bagli exact satir aramak yerine mutable surface, 1:1 copy fast-path, cached scale-map, scaled pixel write ve present davranislarini semantik regex kontratlariyla kontrol eder.
- v0.40.3 one-shot installer ISO lifecycle, v0.40.2 native display telemetry ve v0.40.1 managed download davranislari korunur.


## v0.40.3 One-Shot Installer ISO Session Lifecycle

- Installer ISO `boot_once=true` ile baglandiginda ilk guest boot boyunca bagli kalir.
- Guest `RESET` olayi QMP uzerinden goruldugunde TurkuazVM ISO'yu calisan QEMU'dan live eject eder ve attachment persistence'tan dusurulur.
- VM guest reset olmadan durdurulursa ayni one-shot attachment stop sirasinda otomatik cikarilir.
- Sonraki boot CD/DVD yerine disk konfigurasyonu ile baslar.
- Hypervisor beklenmedik sekilde cikarsa ayni expiry kurali recovery restart oncesinde uygulanir; otomatik recovery tekrar installer ISO'dan baslamaz.
- ISO dosyasinin kendisi silinmez; yalniz VM attachment'i kaldirilir ve kullanici isterse tekrar baglayabilir.
- Kalici CD/DVD boot konfigurasyonlari (`boot_once=false`) otomatik expiry'den etkilenmez.
- v0.40.2 native display telemetrisi ve v0.40.1 managed download duzeltmeleri korunur.


## v0.40.2 Native Display Telemetry + CPU Fallback Render Patch

- Pencere basligindaki eski `FPS / ms` gostergesi ayrildi; `ms = 1000 / FPS` turetimi kaldirildi.
- `RFB UPS`, `Present FPS`, gercek `Render ms`, RAW RFB `Mbps` ve `Drop` sayaci ayri gosterilir.
- Sabit Fedora/Anaconda ekraninda dusuk RFB UPS artik VM grafik performansi gibi yorumlanmaz.
- RFB reader update, rectangle, raw byte, enqueue ve dropped-frame sayaçlarini atomik telemetri olarak tutar.
- Nearest-neighbor CPU scaling icin x/y source mapleri guest veya host boyutu degisene kadar cache edilir.
- Guest ve host boyutu 1:1 ise per-pixel scaling dongusu yerine dogrudan framebuffer kopyasi kullanilir.
- Softbuffer `resize` yalniz render target boyutu degistiginde cagrilir.
- v0.40.1 managed installer download ve mirror failover duzeltmeleri aynen korunur.


## v0.40.1 Managed Installer Download Hotfix

- Linux Mint `official_page_mirrors` resolver kaynagi artik Desktop tarafinda managed download olarak gorulur; Yeni VM, Kurulum Medyasi ve Goruntu Merkezi resolver'i gercekten cagirir.
- Managed download karari UI magic stringi yerine typed domain capability -> Engine API DTO -> Desktop View zincirinden gelir.
- Windows official-page kaynaklari `media_kind=unknown` kaldigi icin otomatik indirmeye acilmaz; resmi sayfa akisi korunur.
- Bir ISO mirrori yarim dosya birakip hata verirse sonraki mirror ayni `.part` dosyasini resume etmez; mirror degisiminde partial temizlenir.
- Ayni ilk mirror yeniden denenirken mevcut resume davranisi korunur.
- Android resolver mimarisi degistirilmedi; v0.40.0 Android CI runtime discovery ve same-build device/host kurali korunur.
- Engine API 24 korunur; yeni `managed_download` alani `serde(default)` ile geriye uyumludur.


## v0.40.0 Official Source Resolver Architecture

- Linux installer medyasi artik sabit katalog URL'sini ana kaynak saymaz; resmi provider indexi runtime'da cozulur.
- Ubuntu, Debian, Fedora, Rocky Linux ve Linux Mint icin typed provider politikalari `config/download-sources.yml` schema 5 altinda tutulur.
- Fedora Server `network_install`, Debian Server `network_install`, Rocky Server `boot`, Ubuntu Server `server_standard` politikasini kullanir.
- Online discovery sonucu last-known-good cache'e yazilir; resmi kaynak gecici olarak erisilemezse uyumlu cache, en son katalog fallback denenir.
- ISO mirror ve checksum mirror failover ayni download Service icinde fail-closed uygulanir; checksum dogrulanmadan medya READY olmaz.
- Runtime discovery daha yeni bir ISO filename bulursa eski managed ISO dosyasi temizlenir ve uygulama yeniden acildiginda dinamik filename READY olarak bulunabilir.
- Android 11-17 Cuttlefish source discovery branch, target, build ve artifact bilgilerini Android CI'dan runtime'da cozer; Android 10 source-build politikasinda kalir.
- Android device image ve host package ayni build kaynagindan cozulur; Android 11 icin kontrollu legacy device-bootloader fallback korunur.
- Android BUILD_INFO machine-readable `raw/BUILD_INFO` endpointinden okunur; device artifact onceligi dokumante edilen `aosp_cf_x86_64_phone-img` ile baslar.
- Linux ve Android HTTP fetch/download islemleri tek `HttpDownloadTool` adaptorunden gecer; curl/retry/timeout ayarlari `downloads.http` altinda tek otoritedir.
- Main config schema 22, download-sources schema 5, Guest Catalog schema 4 ve Engine API 24 korunur.
- v0.39.7 Workspace, v0.39.6 launcher config-drift ve v0.39.5 Engine timeout safety duzeltmeleri korunur.


## v0.39.7 Workspace Layout Hotfix

- `Sanal Makineler > VM Kutuphanesi` yeni workspace listesi artik legacy `machine-grid` CSS gridinden tamamen ayridir.
- Persist edilmis `compact` gorunum tercihi yeni workspace listesini 220px + 360px + action kolonlarina zorlayamaz.
- Fedora 44 Server gibi uzun VM adlari liste icinde dogru yerde baslar, ellipsis ile kontrollu kisalir ve `Konsol/Baslat` hizli aksiyonu panel icinde kalir.
- Sol kutuphane kolonu genisletildi ve 1366/1920 sinifi ekranlarda daha okunabilir olacak responsive olculer tanimlandi.
- 9-10px seviyesine inen kritik VM liste/detay metinleri 10.5-15px araligina cekildi; sekmeler ve durum satirlari daha rahat okunur.
- Genel sekmesi aksiyon kartlari `auto-fit` ile mevcut kart sayisina gore alani doldurur; alt aksiyon butonlari tek bir kontrol olcusune hizalanir.
- Degisiklik yalniz Desktop View/Controller katmanindadir; Engine, Linux media ve Android image akislarina dokunulmaz.

## v0.39.6 Launcher Config Drift Hotfix

- Windows launcher artik `long_request_timeout_ms` icin release'e gomulu eski bir exact deger aramaz.
- `config/turkuazvm.yml` aktif local host degerleri `request_timeout_ms: 5000` ve `long_request_timeout_ms: 300000` olarak kalir.
- Structure gate sayisal timeout iliskisini dogrular; gelecekte timeout politikasi degisse bile verifier eski bir magic string yuzunden acilisi engellemez.
- Kullanici Windows preflight logunda gorulen `DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_MISSING: long_request_timeout_ms: 180000` hatasi bu patch ile kapatildi.



## v0.39.5 Linux Installer Media Policy

- Linux installer medyalari `media_kind` ile siniflandirilir; varsayilan secim Guest Catalog Service tarafinda dagitim/profil politikasina gore yapilir.
- Fedora 44 Server varsayilani DVD yerine resmi Network Install ISO oldu; DVD offline alternatif olarak kalir.
- Rocky Linux 10 Server varsayilani resmi Boot ISO oldu; Minimal ve DVD alternatif olarak kalir.
- Debian 13 Server netinst varsayilani korunur; DVD-1 offline alternatiftir.
- Ubuntu 26.04.1 Server resmi Live Server ISO kullanir; 112 MB netboot tarball ISO olmadigi icin CD/DVD medyasi gibi baglanmaz.
- Desktop profilleri Workstation/Live/Desktop medyayi tercih eder. Linux Mint desktop-only davranisi korunur.
- Guest catalog schema 4 ve Linux media policy regression gate eklendi.


## v0.39.3 Android Legacy CI Artifact Probe Hotfix

- Android CI artifact preflight artik raw artifact URL'lerinde HEAD yerine 1-byte range GET probe kullanir.
- Android 11 kanali config-controlled `allow_device_bootloader_fallback` ile same-build host paketi bulunamazsa device archive icindeki x86_64 QEMU bootloader'i kullanabilir.
- Android 12-17 ve 12L icin host paketi zorunlulugu korunur.
- Ubuntu Desktop ve Server resmi ISO kaynaklari ayri media varyantlari olarak korunur.
- `download-sources.yml` schema 3 oldu.

## v0.39.0 Linux Edition Media + Android Provider Router

- Ubuntu Desktop ve Server resmi ISO akislari ayri tutulur.
- Debian 13 Server netinst yanina Live GNOME Desktop eklendi.
- Fedora 44 Workstation, Server DVD ve Server Network Install ayri resmi medyalar olarak kataloglandi.
- Linux Mint 22.3 Cinnamon desktop-only; Rocky Linux 10 Minimal/DVD server-only olarak modellenir.
- ISO checksum parser GNU `hash filename` ve Fedora `SHA256 (filename) = hash` bicimlerini birlikte destekler.
- Android 11-17/12L Cuttlefish CI kullanir; 11-13 legacy x86_64 phone targetini once dener ve device/host artifact preflight uygular.
- Android 10 icin yayinlanmayan canli CI metadata dongusu kapatildi; provider policy `source_build` olarak fail-closed davranir.

## v0.38.0 Unified Download UX

- Android image, Windows ISO ve Linux ISO indirmeleri tek `download_progress_view.js` View bilesenini kullanir.
- Yeni VM wizard, Kurulum Medyasi ve Goruntu Merkezi ayni animasyon/yuzde/byte/hiz/ETA/sure/asama gorunumunu kullanir.
- ISO tarafindaki eski sahte `%45` ve `%88` ilerleme degerleri kaldirildi; toplam boyut yoksa indeterminate animasyon kullanilir.
- ISO hiz ve ETA degeri polling byte farkindan hesaplanir; Android tarafinda Engine telemetry degerleri kullanilir.


## v0.37.9 Android 10 Legacy CI Discovery

Android 10 `aosp-android10-gsi` kanali modern Android CI `targets[]` semasi disinda cevap verdiginde legacy object/nested status yapilari da parse edilir. `aosp_cf_x86_64_phone-userdebug` Android 10 icin ilk x86_64 hedef olarak denenir ve sabit build ID kullanilmaz.

## v0.37.8 Android Download UX Telemetry

- Yeni VM wizard Android image hazirlarken canli asama, gercek download yuzdesi, byte, hiz, ETA ve gecen sure gosterir.
- Toplam byte bilinmeyen CI yanitlarinda sahte oran uretilmez; hareketli indeterminate progress kullanilir.
- Android image kurulumu wizard icinden iptal edilebilir ve mevcut `install.log` tek tikla acilabilir.
- Android CI backend telemetry modeli degistirilmedi; mevcut Engine API progress kontrati View tarafinda yeniden kullanildi.
- v0.37.7 Windows Rust build hotfix ve Android 11 compatibility davranisi korunur.

## v0.37.7 Android 11 Rust Dead-Code Compile Hotfix

- Windows Rust 1.98.0 gercek build testinde `BuildDiscovery.build_info` alaninin artik okunmadigi ve `-D dead-code` nedeniyle derlemeyi durdurdugu tespit edildi.
- `build_info` metni discovery sirasinda SDK/release parsing icin yerel degisken olarak kullanilmaya devam eder; artik gereksiz olarak `BuildDiscovery` struct icinde saklanmaz.
- Android 11 missing-SDK fallback ve legacy target onceligi v0.37.6 davranisiyla aynen korunur.
- Bu hotfix davranis degisikligi degil, compiler-cleanup patch'idir.

## v0.37.6 Android Release Download Wizard + GUI Cleanup

- Yeni VM sihirbazinda secilen Android 10-17/12L release icin uyumlu READY image'lar filtrelenir.
- Image hazir degilse sihirbazdaki tek aksiyon ilgili release image'ini Android CI uzerinden hazirlamayi baslatir ve durumu otomatik yeniler.
- READY image secilen guest release ile uyusmuyorsa listelenmez ve VM'e atanmaz.
- Android CI Cuttlefish target discovery guncel ve legacy x86_64 target adlariyla genisletildi; SDK dogrulamasi korunur.
- Ana Sayfa icindeki tekrar eden `+ Yeni VM` kaldirildi; global sag-ust create aksiyonu tek primary butondur.
- v0.37.3 ile Windows'ta Rust 1.98.0 + MSVC build tamamlanmis ve Desktop GUI gercek hostta acilmistir.

## v0.37.3 Windows Cargo Process Tree Launcher Hotfix

- Windows `cargo build` sonrasi launcher'in process agacinda beklemesi engellendi.
- Engine, Display ve Desktop ayni Cargo build'de derlenir; Desktop executable dogrudan baslatilir.
- Kullanici Windows testinde v0.37.3 Desktop GUI basariyla acildi ve Engine/QEMU sistem sagligi HAZIR gorundu.

## v0.37.1 Windows PowerShell 5.1 Native Process Hotfix

- v0.37.0 GUI ve Workspace UX aynen korunur.
- Windows PowerShell 5.1 native stderr satirlari PowerShell terminating error olarak yorumlanmaz.
- `rustup`, `rustc`, `cargo`, QEMU ve WinGet exit-code tabanli `windows_native_process_tool.ps1` uzerinden calisir.
- Rust toolchain surumu `rust-toolchain.toml` dosyasindan okunur; launcher icinde ayrica magic version tutulmaz.
- Exact toolchain eksikse rustup ile otomatik kurulum denenir.
- Kullanici logunda gorulen `info: syncing channel updates` kaynakli launcher failure bu patch ile kapatilmistir.

## v0.37.0 Workspace UX + Release Hardening

- Yeni Ana Sayfa dashboard'u sistem sagligi, host ozeti ve son VM'leri tek bakista gosterir.
- VM detay alani modal launcher olmaktan cikarildi; Disk, Ag, Erisim, Snapshot ve Gunluk bilgileri inline Workspace icinde gorunur.
- VM renderer `views/vm_workspace_view.js`, Workspace stilleri `views/workspace.css` altina ayrildi.
- Standart Mod sade navigation kullanir; altyapi sayfalari Uzman Modu altinda progressive disclosure ile acilir.
- Windows launcher global Administrator self-elevation davranisini kaldirdi; yalniz gercekten gerekli WHPX/DISM islemi noktasal elevation kullanir.
- Rust toolchain 1.98.0 ile workspace, launcher ve `rust-toolchain.toml` seviyesinde tek otoriteye baglandi.
- Runtime tarafindan uretilen YAML/evidence dosyalarinda eski surum header'i yazilmasini engelleyen merkezi version kaynagi eklendi.
- Artifact Cache URL policy, IPv4-mapped IPv6 ve DNS rebinding risklerine karsi dogrulanan IP'yi `curl --resolve` ile pinler.
- v0.37.0 Workspace Hardening regression kontrati yeni mimarinin eski patch davranislarina geri donmesini engeller.
- `app.js` icinde artik cagrilmayan uc legacy helper kaldirildi; Windows structure verifier moduler View kaynagini da dogrular.
- `VALIDATION.md`, `ROADMAP.md` ve recovery status aktif v0.37.0 release ile yeniden hizalandi.

## v0.36.5 Launcher Download Sources Path Gate Hotfix

- Windows launcher structure gate icindeki eski `android.image.output_root` beklentisi kaldirildi.
- Android image hedef klasoru artik launcher tarafinda da yalniz `config/download-sources.yml -> paths.android_images` uzerinden dogrulanir.
- Installer media ve Artifact Cache hedefleri de exact merkezi path degerleriyle ayni gate altinda dogrulanir.
- `turkuazvm.yml` Android image config'i yalniz runtime/build davranis alanlarini tasir; download hedef path'i tekrar ana config'e eklenmez.
- `test_v0365_launcher_download_sources_path_gate_contract.py` eski `output_root` launcher regresyonunu fail-closed engeller.


## v0.36.4 Engine Download Config Compile Hotfix

- Windows `cargo build` tarafinda `dead_code = deny` nedeniyle build'i durduran uc eski raw config alani kaldirildi.
- Kurulum ISO, Android image ve Artifact Cache hedef klasorlerinin tek otoritesi artik `config/download-sources.yml` dosyasidir.
- `turkuazvm.yml` ve remote example config'lerindeki tekrar eden eski path alanlari kaldirildi.
- Android CI `base_url` degeri `distribution.yml` provenance kaydina yazilir; boylece mirror ile indirilen image'in kaynagi izlenebilir ve `unused variable` warning'i kapanir.
- Launcher pre-build gate ve yeni regression testi eski cift path alanlarinin geri donmesini engeller.

## v0.36.3 Rust Android CI Compile Hotfix

- Windows `cargo build` sirasinda `crates/guest/src/tools/android_ci_distribution_tool.rs` icindeki test import blokunda kalan gecersiz `format!(...)` ifadesi Rust parser hatasi uretiyordu.
- Gecersiz expression kaldirildi ve testin kullandigi `OFFICIAL_ANDROID_CI_BASE_URL` sabiti dogru sekilde import edildi.
- Yeni regression testi yalniz bu satiri degil, tum Rust kaynaklarinda `use { ... }` bloklarina macro/expression sizmasini tarar.
- Launcher/preflight, Android kaynak kanallari ve Goruntu Merkezi davranislari degistirilmedi.


## v0.36.2 Launcher Android CI Source Gate Hotfix

- Launcher artik `ci.android.com/builds/branches` gibi eski bir birlesik URL literalini aramaz.
- Android CI dogrulamasi `config/download-sources.yml`, Engine channel wiring ve Tool icindeki dinamik `/builds/branches/{branch}/` URL olusturma kontratini kontrol eder.
- Mirror/base URL degistirilse bile launcher resmi host literaline bagli kalmaz.
- v0.36.1 Android image schema migration gate korunur.

## v0.36.1 Launcher Android Image Schema Gate Hotfix

- v0.36.0 Android image manifest semasini 3'e cikardi ancak Windows launcher structure gate eski schema 2 sabitini aramaya devam ettigi icin preflight sonrasi Desktop acilisi bloke olabiliyordu.
- Launcher artik repository icindeki guncel image schema degerini dinamik okur; schema 3 veya daha yeni degeri kabul eder.
- Legacy schema 2 ve schema 1 migration sabitleri fail-closed dogrulanmaya devam eder.
- Bu patch Android image indirme kanallari, VM runtime, Turkuaz NAT, ISO, SSH/RDP veya GUI bilgi mimarisini degistirmez.

## v0.36.0 Goruntu Merkezi + Indirme Kaynaklari

- `Goruntuler` artik Android'e ozel degildir; Windows/Linux kurulum ISO katalogu ile Android Sistem Goruntuleri ayni merkezde ayri sekmelerde yonetilir.
- Standart Mod ISO tarafinda tek `Sistem / ISO` secicisi ve tek ana `Indir` veya `Resmi Sayfayi Ac` aksiyonu gosterir; kaynak kartlari, provider/mimari/not ayrintilari Uzman Modu'nda kalir.
- Android 10, 11, 12, 12L, 13, 14, 15, 16 ve 17 icin ayri resmi CI branch/target profilleri tanimlandi.
- Android image kaydi `requested_release` degerini Engine/API/Repo zincirinde tasir; CI sonucu beklenen SDK ile eslesmeden READY duruma gecmez.
- Ozel mirror kullanilabilir; mirror basarisizsa ayara gore Google resmi `ci.android.com` kaynagina fail-safe fallback yapilir.
- `Ayarlar -> Indirmeler ve Kaynaklar` altina kurulum ISO klasoru, Android image klasoru, Artifact Cache klasoru, Android CI mirror URL ve resmi fallback ayari eklendi.
- Standart Mod kaynak ayrintisini gizler; yol secimleri gorunur, mirror ve cache ayrintilari Uzman Modu'nda kalir.
- Indirme kaynak/yol ayarlari `config/download-sources.yml` dosyasinda merkezilestirildi; ana config schema 21 oldu.
- Goruntu Merkezi'nden baslatilan ISO indirmeleri alt Gorev Cubugu tarafindan aktif is olarak izlenir ve tamamlanma/hata sonucu Gorev Merkezi'ne kaydedilir.

## v0.35.0 Android Catalog + Standard Mode UX

- Android guest katalogu Android 10, 11, 12, 12L, 13, 14, 15, 16 ve 17 surumlerini kapsar.
- Yeni VM olusturma katalogunda bu surumler otomatik gorunur; 12L Tablet profili ayri tutulur.
- Standart Mod Android Sistem Goruntuleri ekrani tek `Surum` secici, tek durum ozeti ve tek ana aksiyona indirgenmistir.
- Release kartlari, profil listeleri, uyumluluk rozetleri, image registry, hizli VM atama ve teknik image satirlari Uzman Modu altina tasinmistir.
- Standart Modda VM release uyusmazligi yalniz kisa bir mesajla gosterilir; yanlis image atamasi yine fail-closed engellenir.
- v0.36.0 ile her release icin surume sabitlenmis resmi CI kanal profili eklendi; SDK dogrulama fail-closed uygulanir.


## v0.34.0 Android Surum Merkezi + Gorev Cubugu

- Android VM icin `Medya` aksiyonu artik Windows/Linux ISO penceresini acmaz; dogrudan Android Sistem Goruntuleri workspace'ine yonlendirir.
- Guest Catalog icindeki Android 17, Android 16 ve Android 15 profilleri surum kartlari olarak listelenir.
- Katalogdaki en guncel Android surumu resmi Android CI `latest` Cuttlefish kanaliyla tek tikla tanimlanabilir, indirilebilir, dogrulanabilir ve secili durmus VM'e otomatik atanabilir.
- Daha eski Android profilleri icin yanlis `latest` image otomatik atanmaz; yalniz ayni Android release degerine sahip READY image varsa atama yapilir.
- Gelismis image ID, AOSP build plan ve register araclari Standart Modu kirletmemesi icin Uzman Modu altinda tutulur.
- Sag altta kayan Task Dock yerine, uygulamanin altinda sabit Gorev Cubugu bulunur.
- Gorev Cubugu aktif Android image kurulumlarini, oturum hata sayisini ve son islemi gosterir; tiklandiginda yukariya dogru Gorev Merkezi acilir.
- Android image indirme basladiginda Gorev Merkezi otomatik acilarak uzun suren isin gorunur kalmasi saglanir.

## v0.33.1 Launcher Workspace Gate Hotfix

- v0.33.0 redesign ile kaldirilan `resource-sidebar` artik Windows launcher structure gate tarafindan aranmaz.
- Launcher yeni VM Library + VM Detail Workspace kontratini fail-closed dogrular.
- Bu patch backend, QEMU, NAT, ISO eject veya Connection Center davranislarini degistirmez.

## v0.33.0 Desktop Workspace Redesign

- Ikinci kaynak gezgini kaldirildi; tek global navigation ve iki bolmeli VM workspace kullanilir.
- Sol VM Library yalniz ad, sistem, temel kaynak ozeti ve durum bilgisini gosterir.
- VM secimi listeyi filtrelemez; secilen makine sagdaki VM Detail Workspace icinde acilir.
- Sag workspace Genel, Konsol, Donanim, Disk, Ag, Snapshot, Erisim ve Gunluk baglamlarini tek VM altinda toplar.
- Baslat, Durdur, Konsol ve Baglan primary aksiyonlari VM detay basliginda yer alir; tehlikeli Sil aksiyonu diger yonetim aksiyonlarindan ayrilir.
- Host sagligi 13 ayri dashboard karti yerine kompakt Sistem Durumu yuzeyinde sunulur.
- Depolama, Ag, Goruntuler, Gorevler, Onbellek ve Ayarlar global navigation icinde popup yerine tam workspace sayfasi olarak acilir.
- Oturum islemleri ana sayfayi kaplayan tablo yerine gerektiginde gorunen Task Dock icinde izlenir.
- Ana navigasyon tek tip inline SVG ikon setine gecmistir.
- Standart / Uzman Modu eklendi; teknik alanlar varsayilan gorunumde gizlenir ve istege bagli acilir.
- Create wizard yedi teknik ekrandan uc ana adima indirildi: Sistem, Kaynaklar, Olustur.
- VM ID, disk ID ve NIC ID otomatik uretilmeye devam eder; gelismis ag ayarlari ikincil bolumde tutulur.
- Connection Center, portable QEMU Turkuaz NAT ve installer ISO eject davranislari korunur.

## v0.32.x Uyumluluk

- v0.32.2 SSH/RDP Connection Center davranisi korunur.
- v0.32.1 installer ISO cikarimi ve kalici disk boot davranisi korunur.
- v0.32.0 QEMU User NAT tabanli Turkuaz NAT davranisi korunur.
- Engine API v24 ve VM manifest schema 6 korunur; v0.36.0 ana config schema 21 ile merkezi indirme kaynak dosyasini tanitir.
- v0.32.x `data` klasoru migration gerektirmeden v0.34.0 altinda kullanilabilir.

## Dogrulama

- v0.33.0 Workspace Contract: PASS
- v0.33.1 Launcher Workspace Gate Contract: PASS
- v0.34.0 Android Release + Taskbar Contract: PASS
- v0.36.0 Image Center + Download Sources Contract: PASS
- v0.36.2 Launcher Android CI Source Gate Contract: PASS
- v0.36.1 Launcher Android Schema Gate Contract: PASS
- v0.36.4 Engine Download Config Compile Contract: PASS
- v0.37.0 Workspace Hardening Contract: PASS
- v0.36.5 Launcher Download Sources Path Gate Contract: PASS
- Full Recovery Gate: PASS
- Python Contract Suite: PASS 44/44
- JavaScript syntax: PASS
- Cargo build: Host Rust toolchain gereken ortamda dogrulanmalidir.
