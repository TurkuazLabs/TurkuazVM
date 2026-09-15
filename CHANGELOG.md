# 📄 Dosya Yolu: /turkuazvm/CHANGELOG.md
# 📌 Amac: TurkuazVM surum degisikliklerini kaydeder
# 📌 Modul - Markdown
# Version: 0.41.2
# Aciklama: TurkuazVM surum kilometre taslarini ve v0.41.2 Baglanti Merkezi calisan-VM erisim yamasini listeler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# Changelog


## v0.41.2 - Baglanti Merkezi Calisan-VM Erisim Yamasi

- Calisan QEMU/Turkuaz NAT VM icin SSH/RDP Hazirla aksiyonlari disable edilmek yerine aktif tutulur.
- View, calisan VM'de port yayininin QEMU baslangic ayari oldugunu aciklar ve stop/restart icin kullanici onayi ister.
- Desktop Service ortak TCP erisim hazirlama akisi ile running -> stopped -> publish -> running orkestrasyonunu yapar.
- Publish hatasinda VM'nin onceki calisma durumuna geri donmesi icin yeniden baslatma recovery adimi uygulanir.
- Hazirlama sonrasi UI dashboard/network state'i yeniler ve localhost hedefini Connection Center'a tekrar yukler.
- Eski `prepareSshAccess.disabled = !vmStopped` ve RDP esdegeri kaldirildi.
- v0.41.2 regression contract bu dead-end akisin geri gelmesini fail-closed engeller.


## v0.41.1 - Workspace Kullanilabilirlik Yamasi

- 1920x1080 ekran goruntulerinde kucuk kalan VM kontrol hedefleri ve metinleri genis ekran breakpointinde buyutuldu.
- Hic VM yokken tekrar eden sol liste ve sag detay bos durumlari tek bos-filo onboarding ekraninda birlestirildi.
- Bos-filo ekranina Yeni VM Olustur, Goruntu Merkezi ve 3 adimli ilk kullanim akisi eklendi.
- Ana Sayfaya Yeni VM, Goruntu Merkezi ve VM Kontrol Merkezi hizli aksiyon kartlari eklendi.
- Goruntu Merkezi standart ISO paneli genis iki kolonlu duzene alindi; gereksiz beyaz alan azaltildi.
- v0.41.0 secili VM komut merkezi ve v0.40.14 Android Emulator port config wiring davranislari korunur.
- Yeni v0.41.1 UI regression contract bos-filo, buyuk ekran ve Goruntu Merkezi kontratini fail-closed kilitler.


## v0.41.0 - VM Kontrol Merkezi

- VM detay ekrani onizleme-oncelikli duzenden kontrol-oncelikli komut merkezine tasindi.
- Secili VM ust alaninda Baslat/Konsol, Durdur, Baglan, Donanim, Disk, Ag ve Snapshot kontrolleri tek bakista erisilebilir hale getirildi.
- CPU, bellek, disk ve IPv4 ozeti sticky komut alanina alindi; gereksiz buyuk VM onizleme karti Genel ekranindan kaldirildi.
- Genel sekmesine buyuk tiklama hedefli Hizli Islemler kartlari ve Calisma Hazirligi paneli eklendi.
- Android image eksigi Genel ekranda dogrudan duzeltme aksiyonu olarak gosterilir.
- VM silme aksiyonu ana komut satirindan ayrilarak daha dusuk yanlis tiklama riskli yonetim alanina tasindi.
- Sol VM listesinde yalniz durum noktasi yerine okunabilir durum etiketi ve daha belirgin Konsol/Baslat aksiyonu kullanilir.
- 1180px, 860px ve 620px kirilimlarinda kontrol merkezi responsive olarak yeniden akar.
- v0.41.0 UI regression contract yeni kontrol merkezi DOM/CSS kontratini fail-closed kilitler.
- v0.40.14 Android Emulator port config wiring ve onceki runtime davranislari korunur.


## v0.40.14 - Android Emulator Port Config Wiring Hotfix

- Windows Cargo buildini `-D dead-code` ile durduran SDK console port config alanlari runtime media Tool ayarlarina baglandi.
- `AndroidApplicationService` sabit ADB port araligi yerine merkezi `android.adb.host_port_min/max` configini kullanir.
- `AndroidRuntimeMediaTool` sabit ADB araligi yerine `download-sources.yml` SDK console port araligini kullanir ve ADB/console ciftini checked arithmetic ile dogrular.
- `allow(dead_code)` veya lint gevsetme eklenmedi; workspace fail-closed derleme politikasi korunur.
- v0.40.14 regression testi ve launcher structure gate port configinin Service -> Tool zincirinde tuketildigini kilitler.
- v0.40.13 resmi Android SDK System Image provider, AVD ve WHPX runtime akisi korunur.


## v0.40.13 - Windows Android SDK System Image Provider

- Windows Android 10-17 ana dagitim politikasi `android_sdk` olarak degistirildi.
- Android 10-17 API 29-37 eslemesi merkezi `download-sources.yml` configine alindi.
- System Image ve Android Emulator paketleri resmi Google repository XML kataloglarindan dinamik cozulur; archive filename tahmini yapilmaz.
- `default`, `google_apis`, `google_apis_playstore` variant fallback sirasi eklendi.
- Android image manifest schema v4 `sdk_emulator` runtime kind ve SDK artifact rollerini persist eder; v3/v2/v1 migration korunur.
- VM-ozel AVD ve private userdata runtime planlari eklendi.
- Windows Android VM runtime'i Android Emulator `-accel auto` ve console/ADB port ciftleriyle calisir.
- SDK Emulator runtime icin Turkuaz NAT process binding atlanir; ADB resmi `emulator-<console-port>` serialini kullanir.
- Legacy Android CI/Cuttlefish resolver ve cache yolu fallback olarak korunur.
- Yeni VM Android install `Logu Ac` ve `Iptal` UI handlerlari tamamlandi.
- Engine API v24 korunur; yeni SDK artifact rolleri public API'de mevcut `Other` DTO rolune map edilir.
- Yeni v0.40.13 regression testi Config -> Service -> Tool -> Runtime -> View zincirini fail-closed kilitler.


## v0.40.12 - Windows PowerShell 5.1 Process Collection Hotfix

- v0.40.11 launcher preflightinde `Get-TurkuazNativeProcessesByExecutablePath` icinde gorulen `Bagimsiz degisken turleri eslesmiyor` hatasi kapatildi.
- Generic `System.Collections.Generic.List[object]` process koleksiyonu plain PowerShell array ile degistirildi.
- PowerShell otomatik `$Matches` degiskeni process sonucu tutmak icin kullanilmiyor.
- Exact executable-path guvenligi, stale Engine/Display cleanup ve runtime file-lock gate davranisi korundu.
- Yeni v0.40.12 regression testi PS5.1-safe koleksiyon kontratini kilitler.

## v0.40.11 - Windows Runtime Binary Lock Hotfix

- Windows kullanici testinde Cargo, calisan `target/debug/turkuazvm-desktop.exe` dosyasini yenileyemedigi icin `os error 5 / Erisim engellendi` ile durdu.
- Launcher ayni workspace Desktop processini exact executable path ile algilar ve zaten calisiyorsa gereksiz rebuild yapmaz.
- Desktop yoksa stale Engine/Display processleri yalniz ayni workspace executable pathleri icin temizlenir.
- Runtime exe dosyalari build oncesi file-lock release gate ile dogrulanir.
- Baska TurkuazVM surumlerine ve global Cargo processlerine dokunulmaz.
- v0.40.10 Engine StartVm crash diagnostics korunur.

## v0.40.10 - Engine StartVm Crash Diagnostics Hotfix

- Android image assignment sonrasi `StartVm` sirasinda gorulen generic `os error 10054` semptomu icin Engine request boundary panic containment eklendi.
- Controller request panicleri `engine_request_panic` API failure cevabina cevrilir; panic payload Engine loguna request ID ve elapsed time ile yazilir.
- Desktop, response-read kopmasinda local Engine processi sonlanmissa process exit status, PID, log path ve son log satirlarini hata detayina ekler.
- StartVm mutating requesti otomatik retry edilmez.
- v0.40.9 Android CI numeric build-id compile hotfixi korunur.


## v0.40.9 - Android CI Numeric Build-ID Compile Hotfix

- Windows Cargo buildinde `turkuazvm-guest` icin `E0425: cannot find function numeric_build_id in this scope` hatasi duzeltildi.
- `collect_status_targets` production parserinin cagdigi `numeric_build_id` helperi yeniden eklendi.
- `dead_code = "deny"` korunur; helper normal resolver akisinda aktif olarak kullanilir.
- Launcher verifier numeric build-id helperini de kontrol eder; ayni eksik sembol artik `STRUCTURE_VERIFY_OK` sonrasina sizamaz.
- Stale v0.37.9 compatibility testi mevcut production parser kontratina tasindi.
- Yeni v0.40.9 regression testi helper cagrisi, tanimi ve rakamsal build-id validation kontratini kilitler.

## v0.40.8 - Android CI Launcher Verifier Contract Hotfix

- Windows launcher `STRUCTURE_VERIFY_OK` oncesinde `ANDROID_SOURCE_PROVIDER_MISSING: parse_last_known_good_build` ile kesiliyordu.
- v0.40.7'de bilincli olarak kaldirilan legacy helperi arayan stale `verify_structure.ps1` kontrati production status parser sembolleriyle degistirildi.
- v0.40.0 Android source resolver regression testi de silinmis helperi zorunlu tutmayacak sekilde production parsera tasindi.
- Yeni v0.40.8 regression testi verifier ile historical regression testinde obsolete Android CI helper sembollerinin tekrar zorunlu hale gelmesini engeller.
- FULL recovery gate v0.40.0 Android source-resolver compatibility testini tekrar aktif regression zincirine alir.
- `dead_code = "deny"` korunur; v0.40.7 Rust cleanup geri alinmadi.

## v0.40.7 - Android CI Dead-Code Compile Hotfix

- `turkuazvm-guest` derlemesinde `-D dead-code` ile blokaj yapan 1 sabit ve 5 duplicate legacy parser helperi kaldirildi.
- Android CI modern/legacy status parser testleri yalniz production `parse_status_target_candidates` yolunu kullanacak sekilde birlestirildi.
- `dead_code = "deny"` workspace politikasi aynen korunur; lint bastirma veya `allow(dead_code)` eklenmedi.
- Yeni v0.40.7 regression contract duplicate helperlarin geri donmesini ve normal resolverin test-only parsera baglanmasini engeller.
- v0.40.6 release metadata ve v0.40.5 historical Android CI dynamic target davranisi korunur.

## v0.40.6 - Release Metadata Contract Hotfix

- Windows launcher preflight basarili olduktan sonra `verify_structure.ps1` tarafinda gorulen `RELEASE_ENGINE_API_VERSION_NOT_FOUND` hatasi duzeltildi.
- v0.40.5 `RELEASE_STATUS` dosyasinda yanlislikla atlanan compatibility metadata geri getirildi: Engine API v24, guest-agent protocol v2 ve aktif schema surumleri.
- FULL recovery Python gate release metadata dosyasini, workspace surumunu, authoritative manifest adini, Engine API Rust sabitini ve Android guest-agent protocol resource degerini artik dogrudan capraz dogrular.
- Yeni v0.40.6 regression contract release status alanlari eksikse static gate'in PASS vermesini engeller.
- v0.40.5 Android 11-16 historical CI target discovery davranisi korunur; Android resolver kodu bu hotfixte degistirilmedi.



## v0.40.5 - Android Historical CI Target Discovery Hotfix

- Android 17 calisirken Android 11-16/12L resolverinin `status.json target bulunamadi` ile kesilmesine neden olan exact-target bagimliligi kaldirildi.
- Android CI `status.json` branch basina tek kez okunur; configured exact targetlar once, historical Cuttlefish x86_64 phone userdebug targetlari ikinci sirada dinamik olarak denenir.
- Branch-prefixed `ID` alanlari normalize edilir; `phone_gki*`, `only_phone-*` ve benzeri historical target adlari SDK/artifact dogrulamasi altinda kabul edilebilir.
- BUILD_INFO SDK dogrulamasi korunur; resolver target adi buldu diye yanlis Android surumunu READY yapamaz.
- Hata logu artik ilgili branch status dosyasinda gorulen target adlarindan diagnostik ozet verir.
- Android 12L branch fallback listesine `aosp-android12L-platform-release` ve `aosp-android12L-dev` eklendi; API 32 ana dogrulama olarak korunur.
- v0.40.4 launcher, v0.40.3 ISO lifecycle ve v0.40.2 display performans davranislari korunur.



## v0.40.4 - Launcher Softbuffer Contract Hotfix

- Windows launcher preflight sonrasi `SOFTBUFFER_DISPLAY_CONTRACT_MISSING: buffer[host_row + x]` ile acilisin kesilmesi duzeltildi.
- v0.40.2 renderer optimizasyonundan sonra stale kalan `buffer[host_row + x]` ve `0x00FF_FFFF` exact-source gate'leri kaldirildi.
- Native display structure gate mutable surface buffer, 1:1 framebuffer copy, cached scale-map, scaled pixel write ve softbuffer present davranislarini semantik patternlerle dogrular.
- Yeni v0.40.4 regression testi verifier'in tekrar yerel degisken adina veya kaldirilmis piksel maskesine baglanmasini engeller.
- v0.40.3 installer ISO session lifecycle ile v0.40.2 display telemetry/render davranislari degistirilmedi.



## v0.40.3 - One-Shot Installer ISO Session Lifecycle

- `boot_once=true` installer ISO attachment'i artik ilk guest boot ile sinirlidir.
- QMP `RESET` olayi typed `GuestReset` runtime eventine donusturulur; Service live media eject ister ve persistence'i disk-only boot'a cevirir.
- `stop_vm` guest reset olmadan kapanirsa QEMU kapandiktan sonra ayni one-shot attachment otomatik dusurulur.
- Beklenmedik QEMU `ProcessExited` olayinda expiry recovery kuyrugundan once uygulanir; auto-restart installer'a geri donmez.
- QMP `eject` forced block-device komutu Tool katmaninda uygulanir; Service katmani QMP wire detayini bilmez.
- `boot_once=false` kalici medya konfigurasyonlari korunur.
- Desktop medya aciklamasi manuel eject zorunlulugu yerine otomatik oturum-sonu cikarma davranisini gosterir.


## v0.40.2 - Native Display Telemetry + CPU Fallback Render Patch

- Native display basligindaki yaniltici `FPS / ms` semantigi kaldirildi; render suresi artik gercek `render()` duration olcumunden gelir.
- RFB framebuffer update hizi `RFB UPS`, host present hizi `Present FPS`, RAW transport hizi `Mbps` ve queue kaybi `Drop` olarak ayrildi.
- RFB reader update/rectangle/raw-byte/enqueue/drop sayaçlari atomik telemetry ile renderer'a aktarilir.
- Softbuffer fallback renderer nearest-neighbor x/y source indexlerini boyut degisene kadar cache eder; her pikselde bolme maliyeti kaldirilir.
- 1:1 guest/host boyutunda dogrudan framebuffer copy fast-path eklendi.
- `Surface::resize` her frame yerine yalniz render target boyutu degistiginde cagrilir.
- v0.40.1 installer managed-download ve mirror failover davranisi korunur.


## v0.40.1 - Managed Installer Download Hotfix

- Linux Mint icin v0.40.0'da eklenen `official_page_mirrors` backend resolver yolunun Desktop tarafinda `mode === direct` filtresiyle yanlislikla engellenmesi kapatildi.
- Installer medya modeline `managed_download_supported()` capability eklendi; Engine API `managed_download` bilgisini View katmanina tasir.
- Yeni VM wizard, Kurulum Medyasi ve Goruntu Merkezi yalniz sabit direct URL'leri degil resolver tarafindan yonetilebilen official-page kaynaklarini da indirir.
- Windows official-page kaynaklari otomatik download kapsaminda degildir ve resmi sayfa davranisi korunur.
- Mirror failover sirasinda onceki mirror'dan kalan `.part` dosyasi temizlenir; farkli mirrorlar arasinda yanlis resume kaynakli checksum/download bozulmasi engellenir.
- v0.40.1 regression kontrati managed-download UI erisimi ve mirror partial izolasyonunu fail-closed kilitler.
- Android source resolver kodu degistirilmedi; v0.40.0 same-build Cuttlefish akisi korunur.


## v0.40.0 - Official Source Resolver Architecture

- Linux installer medyasi icin resmi provider runtime discovery katmani eklendi; katalog URL'si ana kaynak olmaktan cikarildi.
- Ubuntu, Debian, Fedora, Rocky Linux ve Linux Mint typed discovery policy ile cozulur.
- Online source resolution -> last-known-good cache -> catalog fallback sirasi Service katmaninda uygulanir.
- Fedora stable-compose filtresi nightly dosyalarin stable release diye secilmesini engeller.
- ISO ve checksum mirror failover eklendi; SHA-256 beklenen degeri buyuk ISO indirmesinden once cozulur ve her mirror sonrasi dogrulanir.
- Dinamik source filename destegi eklendi; yeni point-release/compose ISO'su indirildiginde eski managed ISO temizlenir ve yeniden acilista READY medya bulunur.
- Android CI discovery ayri provider/resolver/cache katmanlarina ayrildi; branch/build/target/artifact secimi runtime'da cozulur.
- Android 11-17 android_ci, Android 10 source_build provider politikasinda kalir.
- Android device image ile `cvd-host_package.tar.gz` ayni build uzerinden resolve edilir; Android 11 kontrollu device-bootloader fallback korunur.
- BUILD_INFO fetch canonical machine-readable `raw/BUILD_INFO` yoluna alindi; documented x86_64 phone artifact onceligi duzeltildi.
- Linux ve Android download/discovery islemleri ortak `HttpDownloadTool` ve `downloads.http` transport politikasini kullanir.
- Artifact Cache schema gate exact main-config degerine bagli olmaktan cikarildi; schema 22 ile gereksiz launch blokaji engellendi.
- v0.39.7 Workspace Layout, v0.39.6 Launcher Config Drift ve v0.39.5 Engine API Timeout Safety davranislari korunur.

## v0.39.7 - Workspace Layout Hotfix

- Yeni `VM Kutuphanesi` listesi legacy `machine-grid compact-view` CSS kolonlarindan ayrildi; Fedora gibi VM adlarinin saga kaymasi ve hizli aksiyon butonlarinin panel disinda kesilmesi kapatildi.
- Yeni workspace listesi DOM seviyesinde yalniz `vm-library-list` sinifini kullanir; eski card/compact gorunum tercihi yeni workspace'e uygulanmaz.
- Persist edilmis eski `compact-view` degeri icin View tarafinda ek izolasyon ve CSS defensive reset eklendi.
- VM kutuphanesi genisligi 360-390 px bandina alindi; 1180 px altinda 330 px responsive kolon kullanilir.
- Genel sekmesindeki aksiyon kartlari sabit 3 kolon yerine `auto-fit` kullanir; 2 kart varsa bos ucuncu kolon birakmaz.
- Alt detay butonlari header aksiyonlariyla ayni 36 px kontrol yuksekligi ve 11 px tipografiye alindi.
- VM listesi, detay aksiyonlari, kaynak ozeti ve sekme metinleri okunabilirlik icin buyutuldu.
- `test_v0397_workspace_layout_contract.py` ile legacy CSS cakismasi regression gate olarak kilitlendi.
- v0.39.6 launcher config-drift, v0.39.5 Engine API timeout safety, v0.39.4 Linux media policy ve v0.39.3 Android legacy CI hotfixleri korunur.

## v0.39.6 - Launcher Config Drift Hotfix

- Windows launcher `verify_structure.ps1` icindeki eski `long_request_timeout_ms: 180000` magic-value gate kaldirildi.
- Launcher artik timeout alanlarinin varligini ve semantigini dogrular: request timeout sifirdan buyuk, long timeout en az 30000 ms ve request timeouttan kucuk olamaz.
- Aktif local profil `request_timeout_ms: 5000` ve `long_request_timeout_ms: 300000` olarak korunur.
- `test_v0396_launcher_config_drift_contract.py` ile config degeri degistiginde launcher'in eski sabit deger yuzunden acilisi engellemesi regression olarak kapatildi.
- v0.39.5 Engine API timeout safety, v0.39.4 Linux media policy ve v0.39.3 Android legacy CI hotfix aynen korunur.

## v0.39.5 - Engine API Timeout Safety

- Engine API client connect, request-write ve response-read hatalarini ayri siniflandirir.
- Yalniz connect asamasinda basarisizlik varsa local Engine auto-start devreye girer.
- Startup readiness artik mutating istegi tekrar tekrar gondermek yerine yalniz `Ping` ile beklenir; asil istek Engine READY olduktan sonra bir kez gonderilir.
- Response read timeout sonrasi mutating request otomatik retry edilmez; cift VM/disk/network islemi riski kapatildi.
- VM Wizard disk, network ve installer-media mutasyonlari long-request timeout politikasina alindi.
- Installer-media polling aktif download job varken Guest Catalog YAML dosyasini tekrar parse etmez.
- Curl progress meter Engine logundan susturuldu; UI dosya boyutundan kendi progress telemetry'sini gostermeye devam eder.
- Engine response-write hatasi request_id ve elapsed_ms ile loglanir.


## v0.39.4 - Linux Installer Media Policy

- Guest Catalog schema 4 ile `linux_media_policies` ve typed `media_kind` eklendi.
- Fedora Server 44 icin Network Install ISO varsayilan, DVD offline alternatif yapildi.
- Rocky Linux 10 icin Boot ISO varsayilan, Minimal ve DVD alternatif yapildi.
- Debian Server netinst ve Ubuntu Server Live Server resmi varsayilanlari policy ile kilitlendi.
- Guest Catalog Service, Linux template medyasini repository metadata politikasina gore normalize eder; yanlis YAML siralamasi bile DVD'yi otomatik varsayilan yapamaz.
- `test_v0394_linux_media_policy_contract.py` regression gate eklendi.

## v0.39.3 - Android Legacy CI Artifact Probe Hotfix

- Android CI raw artifact varlik kontrolu HEAD yerine range GET + HTTP status probe kullanacak sekilde duzeltildi.
- Android 11 icin config-controlled device archive bootloader fallback eklendi; daha yeni Android kanallarinda same-build host package zorunlu kaldi.
- download-sources schema 3 ve Engine wiring guncellendi.
- Linux Desktop/Server resmi medya ayrimi ve checksum akislarina dokunulmadi.

## v0.39.0 - Linux Edition Media + Android Provider Router

- Linux guest katalogu edition-aware hale getirildi: Ubuntu Desktop/Server, Debian Live GNOME/Server, Fedora Workstation/Server, Mint Cinnamon ve Rocky Server.
- Fedora Workstation Live ile Server DVD/Network Install ayri resmi ISO ve checksum kaynaklarina baglandi.
- Installer Media checksum parser Fedora BSD SHA256 formatini destekleyecek sekilde genisletildi.
- Android release-provider policy config schema 2 eklendi; Android 10 source_build, Android 11-17/12L android_ci olarak tanimlandi.
- AndroidDistributionRouterTool Service ile dis dagitim providerini ayirdi.
- Android 11-13/12L legacy x86_64 phone targeti once denenecek sekilde siralandi.
- Android CI secilen build icin device image ve ayni-build cvd-host package HEAD preflight uygular.

## v0.38.0 - Unified Download UX

- Android image ile Windows/Linux ISO indirme progress gorunumlari tek View component altinda birlestirildi.
- Yeni VM, Kurulum Medyasi ve Goruntu Merkezi ayni determinate/indeterminate animasyonu kullanir.
- ISO progress tarafindaki sahte `%45` / `%88` yuzdeleri kaldirildi.
- ISO polling byte farkindan hiz, ETA ve gecen sure telemetry'si uretilir.
- Android 10 legacy CI discovery ve Android 11 BUILD_INFO fallback kontratlari korunur.


## v0.37.9 - Android 10 Legacy CI Discovery

- Android 10 CI status parser modern `targets[]` disinda object-map ve nested legacy semalari destekler.
- Android 10 release-specific x86_64 target sirasi `aosp_cf_x86_64_phone-userdebug` ile baslar.
- Sema taninmazsa hata loguna schema ozeti eklenir; magic/pinned Android CI build ID kullanilmaz.
- v0.37.8 Android Download UX telemetry korunur.


## v0.37.8 - Android Download UX Telemetry

- Yeni VM wizard Android image hazirlama alanina animasyonlu progress paneli eklendi.
- Engine API tarafinda zaten mevcut olan `downloaded_bytes`, `total_bytes`, `bytes_per_second`, `eta_seconds`, `elapsed_seconds`, `stage` ve `log_path` telemetry alanlari wizard View katmanina baglandi.
- Toplam boyut biliniyorsa gercek dosya indirme yuzdesi ve indirilen/toplam boyut gosterilir; toplam boyut bilinmiyorsa sahte yuzde yerine indeterminate animasyon kullanilir.
- Discovery, disk kontrolu, validation, extract, assemble ve finalize asamalari `Asama N/8` olarak gorunur.
- Wizard icinden aktif Android image kurulumu iptal edilebilir ve install logu acilabilir.
- Android image polling 1 saniyeye indirilerek progress geri bildirimi daha akici hale getirildi.
- `Android X indiriliyor...` aksiyonuna kalici busy spinner eklendi; reduced-motion tercihi korunur.
- v0.37.7 Android 11 BUILD_INFO/dead-code hotfix davranisi degistirilmedi.

## v0.37.7 - Android 11 Rust Dead-Code Compile Hotfix

- Windows Rust 1.98.0 real-host build exposed `BuildDiscovery.build_info` as an unread field under `-D dead-code`.
- The unused struct field and initializer assignment were removed instead of suppressing the compiler lint.
- BUILD_INFO content remains a local discovery value and is still parsed for SDK/release validation.
- v0.37.6 Android 11 missing-SDK fallback and legacy target priority are preserved unchanged.



## v0.37.6 - Android 11 BUILD_INFO Compatibility

- Android 11 CI discovery no longer treats missing BUILD_INFO SDK metadata as a hard mismatch.
- Legacy BUILD_INFO keys `ro.build.version.sdk` and `ro.build.version.release` are supported.
- Exact SDK mismatches still fail closed when Google supplies an SDK value.
- Android 11 release-specific branch metadata provides the fallback SDK/release registration when BUILD_INFO omits those fields.
- `aosp_cf_x86_64_phone-userdebug` is prioritized for Android 11 legacy Cuttlefish builds.
- Wide New VM wizard layout from the GUI cleanup is preserved.

## v0.37.4 - Android Release Download Wizard + GUI Cleanup

- Yeni VM sihirbazinda Android image secimi artik tum READY image'lari degil, yalniz secilen Android release ve mimariyle uyumlu image'lari listeler.
- Android release icin READY image yoksa `Android X Image Indir` aksiyonu dogrudan surume sabitlenmis `ensureAndroidRelease` akisini baslatir.
- Sihirbaz Android image indirme durumunu kendi polling dongusuyle izler; `installing`, `ready` ve `failed` durumlari release bazinda gorunur.
- Kullanici `Daha sonra ayarla` secimini bilincli yaparsa READY image otomatik secilerek bu tercih ezilmez.
- Android CI Cuttlefish x86_64 target adaylari `only_phone`, `aosp_current`, `trunk_staging` ve legacy `phone-userdebug` adlariyla geriye uyumlu hale getirildi.
- Android 15 ve 16 icin release dalina ek GSI branch fallback eklendi; mevcut SDK dogrulamasi yanlis release image'inin READY olmasini engellemeye devam eder.
- Ana Sayfa hero alanindaki ikinci `+ Yeni VM` aksiyonu kaldirildi; global sag-ust `+ Yeni VM` tek primary create aksiyonu olarak kalir.
- v0.37.3 Windows Cargo process-tree launcher duzeltmesi ve v0.37.0 Workspace UX korunur.


## v0.37.3 - Windows Cargo Process Tree Launcher Hotfix

- Windows kullanici testinde Rust 1.98.0, QEMU WHPX, structure verify ve Engine/Display `cargo build` gercek Windows hostta PASS oldu.
- `Start-Process -Wait` tabanli checked native command akisi cargo build tamamlandiktan sonra process agacinda bekleme riskine karsi `System.Diagnostics.Process` ile degistirildi.
- Ilk build artik `turkuazvm-engine`, `turkuazvm-display` ve `turkuazvm-desktop` paketlerini birlikte derler.
- Desktop `cargo run` ile ikinci Cargo oturumu acmak yerine `target/debug/turkuazvm-desktop.exe` uzerinden dogrudan baslatilir.
- Desktop erken kapanirsa launcher `DESKTOP_PROCESS_EARLY_EXIT` ile fail-closed davranir.
- v0.37.2 canonical header hotfix, v0.37.1 PowerShell stderr hotfix ve v0.37.0 Workspace UX aynen korunur.

## v0.37.2 - Windows Header Path Validation Hotfix

- `apps/desktop/ui/styles.css` canonical `/turkuazvm/...` teknik header'i geri getirildi.
- Windows `HEADER_PATH_INVALID: /apps/desktop/ui/styles.css` launch blokaji kapatildi.
- Tum teknik metin dosyalarinin path/header kontratini tarayan v0.37.2 regression testi eklendi.
- v0.37.1 native-process hotfix ve v0.37.0 Workspace UX aynen korundu.

## v0.37.1 - Windows PowerShell 5.1 Native Process Hotfix

- Windows PowerShell 5.1 altinda `rustc.exe` stderr `info:` satirlarinin `$ErrorActionPreference = "Stop"` nedeniyle terminating error'a donusmesi kapatildi.
- `tools/windows_native_process_tool.ps1` eklendi; native process stderr/stdout ile exit-code PowerShell hata semantiginden ayrildi.
- `rustup`, `rustc`, `cargo`, QEMU ve WinGet kritik native cagirilari yeni Tool uzerinden calisir.
- Exact Rust toolchain policy launcher icinde hard-coded degil, `rust-toolchain.toml` dosyasindan okunur.
- Gerekli toolchain kurulu degilse rustup ile kontrollu otomatik kurulum denenir.
- `cargo build` ve `cargo run` ilerleme ciktilari stderr kullansa bile launcher artik yanlis failure uretmez.
- v0.37.0 Home dashboard ve inline VM Workspace davranisi degistirilmedi.
- Yeni v0.37.1 Windows Native Process contract eklendi.

## v0.37.0 - Workspace UX + Release Hardening

- Ana Sayfa dashboard'u eklendi; sistem sagligi, host/QEMU durumu ve VM listesi tek calisma alaninda toplandi.
- VM Disk, Ag, Erisim, Snapshot ve Gunluk sekmeleri modal launcher olmaktan cikarilip inline resource gorunumlerine tasindi.
- Workspace renderer ve stilleri `views/` altinda modulerlestirildi.
- Standart/Uzman Mod progressive disclosure navigation davranisi netlestirildi.
- Windows launcher global Administrator elevation davranisi ve artik kullanilmayan managed network probe fonksiyonu temizlendi.
- Rust 1.98.0 toolchain workspace/launcher/compiler gate ile hizalandi ve `rust-toolchain.toml` eklendi.
- Runtime-generated YAML/evidence version header'lari workspace version kaynagina baglandi.
- Artifact Cache SSRF policy IPv4-mapped IPv6 kontrolu ve DNS-resolution pinning ile sertlestirildi.
- Eski regression testlerinin `RunAs`, stale launcher version ve monolit UI tokenlarini zorlayan kisimlari davranis kontratina migrate edildi.
- Yeni v0.37.0 Workspace Hardening contract eklendi.
- `app.js` icinde tek-referans kalmis legacy `getVmFilterLabel`, `androidAssignmentLabel` ve `androidReleaseNumber` helperlari kaldirildi.
- Windows structure verifier `app.js` ile moduler `vm_workspace_view.js` kaynagini birlikte dogrulayacak sekilde guncellendi.
- Aktif Validation, Roadmap ve Full Recovery status dokumanlari v0.37.0 release gercegiyle yeniden hizalandi.

## v0.36.5 - Launcher Download Sources Path Gate Hotfix

- Windows launcher structure gate icindeki stale `output_root: ./data/android-image-builds` ana-config beklentisi kaldirildi.
- Android image hedef dizini `config/download-sources.yml -> paths.android_images` uzerinden exact path ile dogrulanir.
- Installer media ve Artifact Cache path dogrulamalari ayni merkezi kaynak dosyasinda sertlestirildi.
- v0.36.4 ile tamamlanan `download-sources.yml` tek-otorite mimarisi launcher tarafinda da tutarli hale getirildi.
- Yeni regression testi legacy `android.image.output_root` beklentisinin launcher'a geri donmesini engeller.

## v0.36.4 - Engine Download Config Compile Hotfix

- `dead_code = deny` tarafinda Windows cargo build'i durduran legacy `installer_media_download_root`, `artifact_cache.root` ve `android.image.output_root` raw config alanlari kaldirildi.
- ISO, Android image ve Artifact Cache path degerleri yalniz `config/download-sources.yml` uzerinden okunur.
- Local/remote example ana config'lerdeki tekrar eden eski path alanlari kaldirildi.
- Android CI `base_url` distribution provenance metadata'sina yazilarak mirror kaynagi izlenebilir hale getirildi ve unused-variable warning'i kapatildi.
- Launcher pre-build gate ve `test_v0364_engine_download_config_compile_contract.py` cift config/dead-code regresyonunu engeller.

## v0.36.3 - Rust Android CI Compile Hotfix

- `android_ci_distribution_tool.rs` test modulundeki gecersiz `format!(...)` import ifadesi kaldirildi.
- Test modulune gercekte kullanilan `OFFICIAL_ANDROID_CI_BASE_URL` sabiti import edildi.
- Windows `cargo build` asamasinda gorulen Rust parser `expected one of , :: as or } found !` hatasi kapatildi.
- Tum Rust agaci benzer macro/import enjeksiyonlari icin statik olarak tarandi.
- Yeni regression testi, `use { ... }` bloklarinda `format!` benzeri expression enjeksiyonlarini fail-closed engeller.
- v0.36.2 launcher source gate ve v0.36.0 Goruntu Merkezi davranislari korunur.

## v0.36.2 - Launcher Android CI Source Gate Hotfix

- Windows launcher'in eski `ci.android.com/builds/branches` literalini zorunlu tutmasi kaldirildi.
- Launcher artik Android CI Tool'un dinamik base URL + branch URL olusturmasini dogruluyor.
- `config/download-sources.yml` Android CI source/channel yapisi launcher gate'e dahil edildi.
- Engine'in download source -> distribution channel wiring kontrati launcher gate'e dahil edildi.
- Mirror veya resmi fallback kullanimi launcher tarafinda sabit host URL'sine bagimli degil.
- v0.36.1 schema gate ve v0.36.0 Goruntu Merkezi davranislari korunuyor.

## v0.36.1 - Launcher Android Image Schema Gate Hotfix

- Windows launcher structure gate icindeki eski `IMAGE_SCHEMA_VERSION: u16 = 2` sabit beklentisi kaldirildi.
- Launcher Android image repository semasini kaynaktan regex ile okur ve schema 3 veya daha yeni bir semayi kabul eder.
- Schema 2 ve schema 1 migration sabitleri ayrica fail-closed dogrulanir; geriye donuk repository okuma destegi korunur.
- v0.36.0 `requested_release` ve image schema 3 degisikligi korunur; Engine, GUI, NAT, ISO ve Connection Center davranislarina dokunulmaz.
- `test_v0361_launcher_android_schema_gate_contract.py` regression testi eklendi.


## v0.36.0 - Goruntu Merkezi + Indirme Kaynaklari

- `Goruntuler` global sayfasi Android'e ozel olmaktan cikarildi; Kurulum ISO'lari ve Android sekmeleri tek Goruntu Merkezi altinda birlestirildi.
- Guest Catalog icindeki Windows/Linux resmi ISO kaynaklari merkezde kullanilir; Standart Mod tek secici + tek aksiyon gosterir, tam kaynak kartlari Uzman Modu'nda kalir. Direct kaynaklar TurkuazVM icinden indirilir, Microsoft gibi official-page kaynaklari resmi sayfaya yonlendirir.
- Android 10-17 ve 12L icin surum bazli Android CI kanal katalogu `config/download-sources.yml` dosyasina eklendi.
- Android image aggregate/API/repository zincirine `requested_release` eklendi ve manifest schema 3'e cikarildi.
- Android CI adapter secilen release kanalini kullanir, BUILD_INFO SDK degerini beklenen SDK ile dogrular ve uyusmazlikta fail-closed davranir.
- Android CI base URL mirror olarak ayarlanabilir; resmi `ci.android.com` fallback ayri policy ile yonetilir.
- ISO, Android image ve Artifact Cache hedef dizinleri merkezi download-sources config'ine tasindi.
- `Ayarlar -> Indirmeler ve Kaynaklar` GUI'si eklendi; mirror/cache teknik ayrintilari Uzman Modu altinda tutuldu.
- Ana config schema 21 oldu; local ve remote example config'leri yeni downloads kaynak dosyasina baglandi.
- Goruntu Merkezi'nden baslatilan ISO indirmeleri Gorev Cubugu aktif is sayacina dahil edildi ve completion/error polling eklendi.
- `test_v036_image_center_download_sources_contract.py` regression testi eklendi.


## v0.35.0 - Android Catalog + Standard Mode UX

- Android guest katalogu 10, 11, 12, 12L, 13, 14, 15, 16 ve 17 release kapsamina genisletildi.
- Standart Mod Android image workspace tek surum secici, tek durum ozeti ve tek ana aksiyona indirildi.
- Tam release kartlari, profil/uyumluluk ayrintilari, image registry ve hizli VM atama Uzman Modu altinda tutuldu.
- 12L release normalization eklendi; 12.1/12L image metadata ayni release anahtarina indirgenir.
- VM profili ile image release eslesmesi korunur; surum degisikligi image ekraninda sessizce VM profilini degistirmez.
- `test_v035_android_catalog_standard_mode_contract.py` regression testi eklendi.


## v0.34.0 - Android Surum Merkezi + Gorev Cubugu

- Android VM `Medya` aksiyonu ISO kurulum penceresine dusmek yerine Android Sistem Goruntuleri workspace'ine yonlendirildi.
- Android 17/16/15 guest profilleri release kartlari olarak gorunur hale getirildi.
- En guncel katalog surumu icin resmi Android CI latest Cuttlefish kanalindan `Otomatik Indir ve Ata` akisi eklendi.
- Otomatik image atamasi release dogrulamasi ile sinirlandi; eski Android VM profiline latest image sessizce atanmaz.
- Teknik image registry define/build/register arayuzu Uzman Modu altina tasindi.
- Floating Task Dock yerine uygulama altinda sabit Gorev Cubugu ve yukari acilan Gorev Merkezi eklendi.
- Gorev Cubugu aktif image kurulumlarini, hata sayisini, son islemi ve sistem durumunu gosterir.
- `test_v034_android_release_taskbar_contract.py` regression testi eklendi.

## v0.33.1 - Launcher Workspace Gate Hotfix

- Windows launcher preflight sonrasinda v0.33.0 Desktop Workspace tarafindan kaldirilan `resource-sidebar` bileşenini zorunlu tutan eski structure gate kaldirildi.
- Installer media UI kontrati yalniz installer media elemanlarini dogrular; workspace yapisi ayri fail-closed kontrat ile denetlenir.
- Launcher structure verifier artik `vm-library-pane`, `vm-detail-panel`, `task-dock` ve `expert-mode-button` bileşenlerini zorunlu tutar.
- Legacy `resource-sidebar` Desktop HTML'e geri donerse structure gate fail eder.
- `test_v0331_launcher_workspace_gate_contract.py` regression testi eklendi.

## v0.33.0 - Desktop Workspace Redesign

- Uc kolonlu Desktop shell yerine tek global navigation + VM Library + VM Detail Workspace mimarisine gecildi.
- Resource Explorer kaldirildi; VM secimi artik diger VM'leri listeden gizlemiyor.
- VM listesi ad, guest tipi, CPU/RAM ozeti ve duruma indirgenerek bilgi yogunlugu azaltildi.
- Secilen VM icin Genel, Konsol, Donanim, Disk, Ag, Snapshot, Erisim ve Gunluk baglamlari eklendi.
- Baslat/Durdur/Konsol/Baglan aksiyonlari secili VM baglamina tasindi; Sil aksiyonu ayristirildi.
- Host saglik kartlari kompakt Sistem Durumu strip'inde toplandi.
- Depolama, Ag, Goruntuler, Gorevler, Onbellek ve Ayarlar global navigation icinde popup yerine tam workspace sayfasi olarak acilir.
- Oturum islem tablosu gerektiginde gorunen Task Dock modeline tasindi.
- Ana navigation Unicode semboller yerine tek tip inline SVG ikon seti kullanir.
- Standart / Uzman Modu eklendi; mimari, kaynak ID, acceleration ve gelismis filtreler progressive disclosure ile acilir.
- VM create wizard yedi teknik adimdan `Sistem -> Kaynaklar -> Olustur` seklinde uc ana adima indirildi.
- Disk/medya ve gelismis ag ayarlari progressive disclosure ile ikincil bolumlere tasindi.
- v0.32.2 Connection Center, v0.32.1 installer ISO eject ve v0.32.0 portable Turkuaz NAT regresyonlari korunur.
- `test_v033_workspace_contract.py` yeni Desktop bilgi mimarisini fail-closed regression olarak korur.

## v0.32.2 - Connection Center Usability Patch

- Eski komut-kopyalama odakli `Baglan` penceresi Connection Center olarak yenilendi.
- Desktop Tool katmanina guvenli SSH/RDP launcher, TCP probe ve loopback port secici eklendi.
- Desktop Service stopped Turkuaz NAT VM icin SSH 22 ve RDP 3389 yayinini otomatik hazirlar.
- SSH kullanici adi zorunlu `root` varsayilanindan cikarildi ve VM bazinda UI preference olarak saklanir.
- Hazir baglanti hedefleri test edilebilir; SSH terminali ve RDP istemcisi dogrudan acilabilir.
- Running VM'ler Aglar ekraninda goruntulenebilir, ancak mutating network islemleri stopped state disinda kilitlenir.
- Remote Engine Turkuaz NAT loopback baglantilarinin local desktop uzerinden yanlis kullanimi engellendi.


## v0.32.1 - Installer ISO Eject + Persistent Disk Boot

- Core katmanina stopped VM icin idempotent installer medya cikarim use-case'i eklendi.
- ISO baglantisi kaldirildiginda boot sirasi `Disk`, `boot_once` degeri `false` olarak persist edilir.
- Sanal disk ve TurkuazVM'e kopyalanan ISO dosyasi korunur.
- Engine API v24'e geriye uyumlu `EjectInstallerMedia` aksiyonu eklendi.
- Desktop Controller/Service/Tauri invoke zinciri tamamlandi.
- VM kartina stopped durumda kullanilabilen `Medya` aksiyonu eklendi.
- Medya ekranina `ISO Cikar ve Diskten Baslat` onayli aksiyonu ve acik veri-koruma bilgisi eklendi.
- `test_v0321_installer_media_eject_contract.py` yeniden kurulum dongusunu onleyen zinciri regression olarak korur.

## v0.32.0 - Portable QEMU Turkuaz NAT

- `NetworkMode::ManagedNat` runtime hazirligi platform-specific Windows helper yerine `PreparedNetworkBackend::UserNat` uretiyor.
- QEMU command builder managed IPAM bilgisinden `net=<subnet>/<prefix>`, `host=<gateway>` ve `dhcpstart=<vm-ip>` argumanlarini olusturuyor.
- Managed NAT port publish kayitlari QEMU `hostfwd` ile calisiyor.
- Windows launcher varsayilan ag icin OpenVPN/TAP dependency bootstrap ve fail-closed zorunlulugunu kaldirdi.
- TAP/Bridge altyapisi Advanced Network uyumlulugu icin korunuyor.
- Linux hostta `managed_nat` capability aktif hale getirildi.
- `test_v0320_qemu_user_nat_contract.py` portable NAT kontratini regression olarak koruyor.
- Linux root launcher `TurkuazVM-Start.sh` ve `scripts/start_turkuazvm_linux.sh` eklendi.
- QEMU NAT service publish varsayilan bind adresi `127.0.0.1` yapildi.
- Desktop `Baglan` akisi QEMU NAT icin hostfwd SSH/RDP endpointlerini kullanacak sekilde duzeltildi.
- v0.31.x stale ManagedTap lease cleanup hatasi portable-only VM startini artik bloklamiyor.

## v0.31.2 - Windows Managed Network Transaction Hotfix

- `ensure` asamasi transactional rollback ile sarildi; yeni bridge/TAP kalintilari hata sonrasi temizlenir.
- State yazilmadan yarida kalan Turkuaz bridge adapterleri ad + InterfaceGuid veya guvenli tek-bridge membership sinyaliyle recovery edilir.
- Bridge create ilk denemesi reddedilirse Turkuaz TAP adapterlerine `forcecompatmode=enable` uygulanip tek kontrollu retry yapilir.
- `tapctl create/delete` ciktilari exit code ile kalici network loguna yazilir.
- `probe` aksiyonu gercek `tap0901` create/delete testi yapar ve `netsh bridge show adapter` yetenegini dogrular.
- Windows launcher `Turkuaz NAT Runtime` preflight sonucu gosterir ve TAP driver/bridge runtime probe basarisizsa Desktop baslatmayi fail-closed durdurur.
- `test_v0312_windows_network_transaction_contract.py` yeni rollback/recovery/probe zincirini regression olarak korur.

## v0.31.1 - Windows Managed Network Hotfix

- Windows managed network helper icindeki brace edilmemis `variable:` PowerShell interpolation parse bugi giderildi.
- Bridge olusturma sonrasi sabit 1 saniyelik adapter discovery yerine retry + InterfaceGuid tabanli cozumleme eklendi.
- Bridge uyeligi locale-dependent `Yes` metni yerine `ms_bridge` binding uzerinden kontrol edilir.
- Helper stage loglari `data/logs/network/windows-managed-network.log` altina yazilir.
- Rust helper runner stdout/stderr'i artik yutmaz; `vm_start_failed` hatasinda gercek Windows nedeni `detail=` ile gorunur.
- Baska bir host WinNAT varliginda `TURKUAZ_WINNAT_CONFLICT` explicit diagnostic doner.
- `test_v0311_windows_managed_network_hotfix.py` ayni PowerShell parse sinifi ve diagnostic zincirinin geri gelmesini engeller.

## v0.31.0 - Full Step-by-Step Create Wizard

- `Sanal Makine Olustur` uzun tek sayfa modelinden gercek yedi adimli wizard modeline tasindi.
- Akis `Sistem -> Kimlik -> Donanim -> Disk -> Medya -> Ag -> Ozet` olarak ilerler ve ayni anda tek adim gorunur.
- `Ileri` mevcut adimi validate eder; `Geri` girilen degerleri koruyarak onceki ekrana doner.
- Disk boyutu ve kaynak ID plani final create oncesi secilir.
- Medya adimi resmi ISO download/cancel, yerel ISO ve Android READY Image secimini destekler.
- Ag profili final create oncesi Turkuaz NAT, Private, Bridge veya Legacy User NAT olarak secilir.
- `VM Olustur` yalniz final Ozet ekraninda gorunur.
- Final submit VM, disk, network ve medya/image islemlerini sirali uygular; hata durumunda draft kaynaklar best-effort rollback ile temizlenir.
- `test_v031_step_by_step_create_wizard_contract.py` tam wizard ve orchestration zincirini fail-closed korur.

## v0.30.0 - Premium ServBay Create Wizard

- Sanal Makine Olustur ekranindaki sol akis rail'i tiklanabilir hale getirildi.
- Katalog, Kimlik, Kaynak ve Ozet bolumleri focus durumuna gore canli vurgulanir.
- Rail uzerinden ilgili create bolumune yumusak kaydirma eklendi.
- OS, ID, CPU ve GO compact baslik ikonlari eklendi.
- Dropdown, kaynak karti ve ozet spacing degerleri daha yogun ServBay duzenine cekildi.
- Create modal header ve action bar sticky hale getirildi.
- `Vazgec` ve `VM Olustur` aksiyonlari uzun wizard ekraninda gorunur kalir.
- `test_v030_create_wizard_premium_contract.py` ile yeni etkilesimler regression kapsamina alindi.

## v0.30.0 - ServBay Create Wizard

- `Sanal Makine Olustur` akisi ServBay benzeri iki kolonlu kompakt wizard duzenine tasindi.
- Sol rail ile Katalog, Kimlik, Kaynak ve Sonraki Adim bolumleri daha okunakli hale getirildi.
- Canli secim ozeti aile, dagitim, surum, profil, mimari ve firmware alanlarini anlik gosterir.
- Kaynak profili vCPU, RAM ve onerilen disk kartlariyla daha sade gosterilir.
- Sonraki adim plani create ekraninda onceden gosterilerek Disk -> Medya/Image -> Ag akisinin anlasilmasi kolaylastirildi.
- `test_v029_create_wizard_servbay_contract.py` ile yeni create flow yapisi regression kapsamina alindi.

## v0.30.0 - ServBay Navigation + Installer Download Control

- Desktop UI ServBay esintili global sol menu, ikinci kaynak gezgini ve sag calisma alanindan olusan uc kolonlu shell'e tasindi.
- Sidebar koyu Proxmox gorunumunden acik, kompakt servis yonetim navigation stiline gecirildi.
- VM listesinde kompakt gorunum yeni varsayilan oldu; mevcut kart gorunumu kullanici secenegi olarak korundu.
- Kurulum Medyasi modalina `Indirmeyi Durdur` butonu eklendi.
- Engine API 24 `CancelInstallerMediaDownload` action'i, Desktop Controller ve Desktop Service zinciri eklendi.
- Installer media downloader `curl` child process'ini polling ile yonetir; cancel isteginde process sonlandirilir ve yarim `.part` dosyasi temizlenir.
- SHA-256 dosya okuma dongusu cancellation-aware hale getirildi; `cancelling` ve `cancelled` durumlari UI'ya tasindi.
- `test_v028_servbay_download_cancel_contract.py` katmanlar arasi iptal ve yeni shell contractini fail-closed korur.
- Config schema 20, VM manifest schema 6, Guest Catalog schema 3 ve Guest Agent protocol 2 degismedi.

## v0.27.1 - Launcher Network Verifier Hotfix

- Windows launcher preflight sonrasinda `verify_structure.ps1` tarafindan uretilen `STORAGE_NETWORK_MAIN_CONFIG_MISSING: default_network_id: default` false-negative blokaji giderildi.
- Structure verifier `default_network_id: turkuaz-net-01`, `private_network_id: turkuaz-private-01`, `default_profile: managed_nat` ve Windows managed helper kontratini dogrular.
- `test_v0271_launcher_network_verifier_contract.py` stale `default` ag beklentisinin geri gelmesini fail-closed engeller.
- Engine API 23, config schema 20 ve VM manifest schema 6 degismedi.

## v0.27.0 - Managed Network + Connection UX

- Varsayilan `Turkuaz NAT` profili ile Windows managed TAP fabric, Network Bridge, WinNAT, DHCP reservation ve IPAM zinciri eklendi.
- Managed NAT VM'leri private IPv4/gateway/DNS/MAC bilgisini manifest schema 6 ile kalici saklar.
- Host route tablosuna gore `192.168.240.0/24` ve alternatif private subnet adaylari cakisma kontrolunden gecirilir.
- `Private`, `Bridge / Existing TAP` ve `Legacy User NAT` ag profilleri typed domain/API kontratlarina eklendi.
- SSH/HTTP/HTTPS/custom servis yayinlama commandlari ve WinNAT static mapping runtime entegrasyonu eklendi.
- Windows launcher OpenVPN TAP `tapctl` bagimliligini bootstrap/preflight zincirine dahil eder ve managed network icin yonetici yetkisiyle yeniden baslar.
- QEMU 2D/VirGL x86 display bootstrap `virtio-vga` / `virtio-vga-gl` olarak degistirildi; `Display output is not active` installer scanout problemi hedeflendi.
- Desktop VM kartinda `Konsol`, managed IP icin `Baglan`, SSH/RDP baglanti bilgileri ve ag servis yonetimi eklendi.
- VM Creation Wizard buyuk OS kartlari yerine 5 sirali dropdown ve acilir donanim bolumu kullanir.
- Sol navigasyon Compute / Altyapi / Sistem gruplarina ayrildi.
- Engine API 23, config schema 20, VM manifest schema 6, Guest Catalog schema 3 ve Guest Agent protocol 2.

## v0.26.0 - Host-Aware Installer Media Center

- Guest Catalog schema 3 ile primary installer media yanina `installer_media_options` alternatifleri eklendi.
- Installer media kaynaklari benzersiz `media_id`, hedef mimari ve `recommended` bilgisi tasir.
- Debian 13 Server icin Debian 13.6 amd64 netinst resmi cdimage kaynagindan dogrudan indirme ve SHA256SUMS dogrulamasi eklendi.
- Debian 13.6 DVD-1 ve Debian 12.15 oldstable netinst alternatif medya secenekleri eklendi.
- Ubuntu template'leri icin uyumlu 26.04.1 ve 24.04.4 resmi ISO alternatifleri eklendi.
- Desktop medya merkezi host ve VM mimarisini karsilastirir; uyumsuz medya seceneklerini pasiflestirir ve onerilen uyumlu medyayi otomatik secer.
- `Farkli Surum / Medya Sec` ile kullanici ayni VM template'i icin baska resmi medya varyantini otomatik indirebilir.
- Download API `media_id` tasir ve her varyanti ayri data klasorunde izole eder.
- Engine API 22, Guest Catalog schema 3, config schema 19 ve Guest Agent protocol 2.
- FULL paket `docs/versions/` alani aktif surumle sinirlandi; v0.13-v0.15 ara faz dokumanlari release paketinden temizlendi ve regression gate ile tekrar sizmasi engellendi.


## v0.25.3 - Installer Media Reconfigure Hotfix

- `media/installer.iso` mevcutken ayni medya adiminin tekrar calistirilmasi `Media(AlreadyExists(...))` ile durmaz.
- MediaPort import sonucu `Created`, `Unchanged` veya `Replaced` olarak izlenir.
- Ayni ISO SHA-256 karsilastirmasi ile no-op olarak kabul edilir.
- Farkli ISO atomik temp + backup modeliyle replace edilir.
- Guest boot hazirlama veya repository persistence basarisiz olursa onceki ISO rollback ile geri yuklenir.
- LocalGuestMediaTool icin idempotent import, replace commit ve replace rollback unit testleri eklendi.
- Engine API 21, config schema 19, Guest Catalog schema 2 ve Guest Agent protocol 2 degismedi.


## v0.25.2 - Resource Naming Collision Hotfix

- VM kimligi onerileri mevcut VM ID listesini kontrol eder; ayni sablon tekrar secilirse `-2`, `-3` seklinde benzersiz suffix uretilir.
- Makine adi kullanici tarafindan degistirilirken VM ID elle kilitlenmemisse ID otomatik olarak ada gore guncellenir.
- Disk ID varsayilani `system` sabitinden cikarildi; secili VM icin `<vm-id>-disk-01`, `<vm-id>-disk-02` sirasi otomatik uretilir.
- Ag attachment ID varsayilani `default` sabit UI davranisindan cikarildi; `<vm-id>-net-01`, `<vm-id>-net-02` sirasi kullanilir.
- Desktop -> Controller -> Service -> Engine API zinciri network ID degerini tasir; Engine API v21 oldu.
- Eski VM disk/ag kimlikleri geriye donuk uyumluluk icin degistirilmez.
- `test_resource_naming_contract.py` regression testi ve structure gate tokenlari eklendi.

## v0.25.1 - Release Root Cleanup

- Eski `RELEASE_STATUS_v*.yml` ve `MANIFEST_SHA256_v*.txt` kopyalari aktif FULL paket kokunden kaldirildi.
- H-L donemi `RELEASE_GATE_STATUS`, cumulative manifest, `PATCH_INTEGRATION` ve `VALIDATION` root artefactlari kaldirildi.
- Recovery baseline `docs/recovery/FULL_RECOVERY_STATUS.yml` altina tasindi ve guncel surumle yenilendi.
- Runtime validation gate tarihsel K-L supersession evidence dosyalarina bagimli olmaktan cikarildi; aktif config, katman, qemu-img, language ve current release metadata invariantlarini denetler.
- `docs/releases/RELEASE_ARTIFACT_POLICY.md` ile release artefact saklama politikasi belgelendi.
- Launcher structure gate stale root artefactlarini fail-closed reddeder.
- Engine API `20`, config schema `19`, guest catalog schema `2`, Guest Agent protocol `2` degismedi.

## v0.24.1 - Local Log Viewer Structure Gate Hotfix

- `verify_structure.ps1` local log viewer kontrati generic `.log/.txt` data-root viewer davranisina gore semantik hale getirildi.
- Android `install.log` literal kontrolu generic viewer dosyasindan kaldirildi; Android install log ownership kontrolu `local_log_catalog_tool.rs` tarafina tasindi.
- `LOCAL_INSTALL_LOG_VIEWER_MISSING: install.log` false-negative launcher blokaji giderildi.
- Local viewer path confinement, canonicalize, file check ve Windows viewer process kontratlari korunur.
- `test_local_log_viewer_contract.py` regression testi eklendi.
- Engine API `19`, config schema `18`, Guest Agent protocol `2` degismedi.

## v0.24.0 - UI Completeness and Installer Media

- Engine API v19 ile disk buyutme/silme, VM duzenleme/silme ve installer ISO baglama commandlari eklendi.
- Config schema 18 ile guest installer media ve firmware ayarlari Engine config katmanina baglandi.
- Depolama ekranina secili VM icin bagli disk listesi, buyutme ve silme aksiyonlari eklendi.
- Ag ekranina bagli NIC listesi ve mevcut `detach_vm_network` backend commandina bagli kaldirma aksiyonu eklendi.
- Windows/Linux/Diger post-create akisi `VM -> Disk -> Kurulum Medyasi -> Ag -> Ozet` olarak tamamlandi.
- Windows hostta native ISO secici eklendi; ISO secimi Tool katmaninda izole edildi ve GuestBootService uzerinden VM media alanina import edilir.
- Launcher, Engine ve Android image install loglarini listeleyen merkezi `Gunlukler` menusu eklendi.
- VM kartina stopped durumda ad/vCPU/RAM duzenleme ve stopped/error durumda guvenli silme aksiyonlari eklendi.
- Error VM silme oncesi lifecycle recovery uygulanir; VM repository silme machine klasorunu ve alt kaynaklarini kalici kaldirir.
- VM summary DTO disk, ag ve installer media ayrintilarini Desktop yonetim ekranlarina tasir.
- Eski gizli `create-assign-image` ve `create-add-network` bypass kontrolleri ve handlerlari tamamen kaldirildi.
- `test_ui_completeness_v024.py` ile gorunen yonetim kontrollerinin API/Engine/Controller/Service/View zinciri fail-closed dogrulanir.
- Engine API `19`, config schema `18`, Guest Agent protocol `2`.

## v0.23.4 - Post-Create Storage Wizard Flow Hotfix

- VM olusturma sonrasi Storage penceresi yalniz post-create akista wizard moduna gecebilir.
- Disk olusturulmadan `Ileri` pasif kalir; basarili disk olusturma sonrasi aktif olur.
- Android template akisi `Ileri: Android Image` ile hedef VM secili Android Image Manager'a gider.
- Windows/Linux/Diger template akisi `Ileri: Ag` ile hedef VM secili Network yapilandirmasina gider.
- Post-create Storage akisi hedef VM secimini kilitler; yanlis VM'ye disk ekleyip akisi ilerletme riski azaltildi.
- Hedef VM'de zaten disk varsa wizard yeniden disk olusturmaya zorlamadan `Ileri` aksiyonunu aktif eder.
- Normal Depolama menusu wizard footer/progress gostermeden eski genel Storage davranisini korur.
- Disk olusturma sonrasi otomatik modal gecisi kaldirildi; sonraki adim kullanicinin `Ileri` aksiyonuna baglandi.
- Post-create storage flow regression contract ve structure gate tokeni eklendi.
- Engine API 18, config schema 17 ve Guest Agent protocol 2 degismedi.

## v0.23.3 - Local Engine Supervision and Diagnostics Hotfix

- Desktop tarafindan baslatilan local Engine process artik `EngineProcessHandle` ile izlenir.
- Engine beklenmedik kapandiginda sonraki API istegi local Engine'i kontrollu olarak yeniden baslatabilir.
- Engine stdout/stderr artik `data/logs/engine/` altinda kalici log dosyasina yazilir; stderr artik null'a atilmaz.
- Startup sirasinda Engine kapanirsa Desktop PID, exit status, log yolu ve log tail bilgisini hata mesajina ekler.
- Engine process calisiyor ancak API hazir olmuyorsa hata mesaji endpoint, PID ve log yolunu gosterir.
- Varsayilan local Engine startup timeout 8 saniyeden 15 saniyeye cikarildi.
- Engine supervision regression contract testi ve structure gate tokenlari eklendi.
- Engine API 18, config schema 17 ve Guest Agent protocol 2 degismedi.

## v0.23.2 - Android Image Readiness UX Hotfix

- Android VM start readiness disk ve image assignment kosullariyla Desktop tarafinda onceden dogrulanir.
- `PendingFirstBoot` assignment ilk boot icin gecerlidir; UI yanlis sekilde `Ready` state beklemez.
- Image assignment olmayan Android VM kartina `Image Ata` yonlendirmesi eklendi.
- Bulk Start image assignment olmayan Android VM'leri secmez.
- Post-create Android akisi VM -> Disk -> Android Image -> Baslat siralamasina getirildi.
- Disk olusturma post-create Android VM icinse Android Image Manager hedef VM secili olarak acilir.
- Android image atama islemleri oturum aktivitesine ve toast bildirimine yansitilir.
- Android start readiness regression testi eklendi.
- Engine API 18, config schema 17 ve Guest Agent protocol 2 degismedi.

## v0.23.1 - Guest Boot Domain Error Compile Fix

- `VmLifecycleError` guest boot validation errorsini `GuestDomain(GuestBootDomainError)` ile ayri tasir.
- `GuestBootConfiguration::default_for_profile` sonucu artik dogru error ailesine map edilir.
- Windows Rust E0631 `GuestBootDomainError -> VmDomainError` function signature mismatch giderildi.
- Invalid catalog template ID icin unit regression testi eklendi.
- Engine API 18, config schema 17 ve Guest Agent protocol 2 degismedi.

## v0.23.0 - Guest Catalog and VM Creation Wizard

- YAML tabanli typed Guest Catalog bounded context eklendi.
- Windows, Linux, Android ve Diger OS aileleri icin cascading product/release/profile secimi eklendi.
- Katalog kaynakli vCPU, RAM, disk, firmware ve source metadata onerileri eklendi.
- Engine API v18 `ListGuestCatalog` ve `GuestTemplateDto` contractlarini sunuyor.
- Config schema 17 `guest_catalog.path` ile katalog kaynagini merkezi config'e bagliyor.
- `guest_template_id` VM guest boot manifestinde kalici saklaniyor.
- Engine template ID'yi server-side katalogdan tekrar cozer; guest profilinin UI tarafinda sahte/deger kaymasina guvenmez.
- VM olusturma sonrasi Disk Ekle / Varsayilan Ag Ekle akisi eklendi.
- Android template sonrasi Android Image Manager hizli atama akisina yonlendirme eklendi.
- Guest Catalog Wizard regression gate eklendi.


## v0.22.1 - VM Readiness and Persistent Navigation Hotfix

- Disk bulunmayan VM icin `Baslat` ve toplu baslatma engellenir; runtime `DiskBootRequiresDisk` hatasina dusmeden readiness uyarisi gosterilir.
- Disk eksik VM kartina `Disk Ekle` aksiyonu eklendi; Storage penceresi hedef VM secili olarak acilir.
- Disk ekleme sirasinda HypervisorStart kaynakli Error state guvenli lifecycle recovery ile Stopped durumuna alinabilir.
- `DiskBootRequiresDisk` teknik hata metni kullanici kartinda tekrar edilmez; kullaniciya `Onyukleme diski gerekli` aciklamasi sunulur.
- Sol navigasyon 100vh icinde kendi scroll alanina alindi; ana content ayri scroll olur ve sticky toolbar kaybolmaz.
- Desktop UX regression gate readiness, disk aksiyonu ve persistent navigation tokenlarini kontrol eder.
- Workspace package version `0.22.1`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.


## v0.22.0 - Desktop UX Operations Dashboard

- Proxmox resource tree/search/task log desenleri TurkuazVM mevcut backend yeteneklerine uyarlanarak eklendi.
- Dashboard gercek VM verisinden Toplam, Calisiyor, Durdu, Hata, Snapshot ve Disk fleet ozetlerini hesaplar.
- Hizli filtreler Android/Linux/Windows ve runtime state secimlerini destekler.
- VM listesi ad, durum ve profile gore siralanabilir; Kart ve Kompakt gorunum kullanici tercihinde saklanir.
- Gorunen VM seti icin toplu Baslat ve Durdur eklendi; toplu durdurma onay gerektirir.
- Bu oturumda baslatilan VM, disk, ag, snapshot ve clone islemleri Oturum Islemleri tablosunda izlenir.
- Basari/hata toast geri bildirimi, busy button durumu ve F5/Ctrl+K/Ctrl+N/Esc kisayollari eklendi.
- Sol panel daraltma tercihi localStorage ile korunur.
- Islevi olmayan dekoratif kontrol eklenmedi; yeni kontroller mevcut Tauri/Engine commandlari veya local UI state ile gercek davranisa baglidir.
- Desktop UX structure gate semantik tokenlarla genisletildi.
- Workspace package version `0.22.0`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.


## v0.21.8 - Desktop Theme Contract Hotfix

- Proxmox esintili TurkuazVM desktop arayuzu koyu sol navigasyon, ust komut cubugu, durum kartlari ve yeni VM kartlari ile uygulandi.
- `scripts/verify_structure.ps1` artik tema renklerini exact hex/RGBA degerlerine kilitlemez.
- Theme gate `color-scheme`, `--panel`, `--text`, `--accent`, `--sidebar-bg`, `.sidebar`, `.workspace-panel` ve `.metric-card` semantik kontratlarini dogrular.
- `DESKTOP_LIGHT_THEME_CONTRACT_MISSING: --text: #18302d` false-negative blokaji giderildi.
- Workspace package version `0.21.8`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.


## v0.21.7 - Windows QEMU Process Identity Hotfix

- Windows QEMU process identity sorgusunda `powershell.exe -Command <script> <pid>` kullanimi PID degerini PowerShell kodunun devamina ekleyip `Unexpected token '<pid>'` parser hatasi uretiyordu.
- PID artik Rust tarafinda `u32` olarak tek PowerShell command stringine guvenli bicimde render edilir; `-Command` sonrasinda ikinci positional token gecilmez.
- QEMU start -> QMP ready -> process identity -> runtime registry zinciri Windows'ta tekrar calisabilir hale getirildi.
- Android CI v0.21.6 READY bundle sonucu korunur; config schema `16`, Engine API `17`, Guest Agent protocol `2` degismez.


## v0.21.6 - Windows Cuttlefish Host Archive Extraction Hotfix

- Windows `tar.exe` full Cuttlefish host extraction sirasinda gorulen `Can't create \\?\...` hatasi kullanici loguyla dogrulandi.
- `cvd-host_package.tar.gz` artik Windows-native provisioning sirasinda tam extract edilmez.
- TurkuazVM'in gercekte ihtiyac duydugu x86_64 QEMU bootloader archive listing icinden secilir ve `tar -xOzf` stdout stream ile bundle'a yazilir.
- Linux-only `launch_cvd`, `stop_cvd` ve host tool agaci Windows filesystemine materialize edilmez.
- x86_64 secici AArch64/RISC-V/ARM adaylarini reddeder ve deterministik rank uygular.
- Workspace package version `0.21.6`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.


## v0.21.5 - Android CI Public Status Discovery Hotfix

- Kullanici Windows logunda legacy Android CI discovery endpointinin `Error 403: Rate limit exceeded for legacy API` ile kapandigi dogrulandi.
- Latest build discovery `ci.android.com/builds/latest/branches/.../view/BUILD_INFO` yerine resmi public `ci.android.com/builds/branches/<branch>/status.json` akisina tasindi.
- LKG parser `targets[].ID` ve `targets[].name` semalarini, string veya numeric `last_known_good_build` degerini destekler.
- Artifact base URL numeric build ID ile dogrudan `/submitted/<build_id>/<target>/latest` olarak uretilir.
- Concrete `BUILD_INFO` metadata best-effort hale getirildi; metadata erisimi basarisiz olsa bile device/host artifact kurulumu bloke edilmez.
- Build API v4 icin son kullanicidan Google OAuth/service-account credential istenmez.
- Workspace package version `0.21.5`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.

## v0.21.4 - Android CI 403 Transport Resilience Hotfix

- Windows Android CI `BUILD_INFO` discovery isteginde gorulen HTTP 403 icin browser-compatible request identity eklendi.
- Discovery once headerli `curl` ile denenir; basarisiz olursa Windows PowerShell `Invoke-WebRequest -UseBasicParsing` fallback'i ayni resmi CI URL'sini kullanir.
- Device image, host package ve Content-Length HEAD istekleri ayni Android CI request header politikasini kullanir.
- Android install log, install-status, cancel marker ve distribution provenance dosyalarindaki runtime `Version: 0.12.4` sabitleri `CARGO_PKG_VERSION` ile dinamik hale getirildi.
- `turkuazvm-artifact-cache` derlemesindeki kullanilmayan `ArtifactSourceValidators` import warning'i temizlendi.
- Workspace package version `0.21.4`; config schema `16`, Engine API `17`, Guest Agent protocol `2`.

## v0.21.3 - Android ADB Guard Structure Gate Hotfix

- `scripts/verify_structure.ps1` icindeki stale `android.adb.host_ip must be loopback in v0.12.4` beklentisi kaldirildi.
- Verifier ADB loopback, remote native display ve ARM-translation policy kontrollerinde release numarasi yerine stabil semantik contract arar.
- Guest Agent protocol `v1` stale gate kaldirildi; release metadata, Rust TVGB contract, Android Kotlin secure-channel ve resource degeri birlikte dogrulanir.
- Android distribution retry gate eski `CURL_RETRY_COUNT` sabiti yerine `download_retry_count`, retry-delay ve connect-timeout alanlarini dogrular.
- TurkuazDisplay/Desktop console surumu `CARGO_PKG_VERSION` uzerinden dinamik gosterilir.
- Display, Desktop ve AOSP builder hata/not metinlerindeki stale `v0.12.4` release branding temizlendi.
- Workspace package version `0.21.3`; config schema `16`, Engine API `17`.

## v0.21.2 - Engine API Structure Gate Hotfix

- `scripts/verify_structure.ps1` icindeki stale `ENGINE_API_VERSION = 14` beklentisi kaldirildi.
- Engine API contract version artik aktif `RELEASE_STATUS_v<workspace-version>.yml` metadata ile karsilastiriliyor.
- Launcher branding workspace package version alanindan dinamik okunuyor.
- Regression gate ve FULL recovery summary patch surum magic string bagimliligindan cikarildi.
- Workspace package version `0.21.2`; config schema `16`, Engine API `17`.

## v0.21.1 - Windows Launcher Structure Gate Hotfix

- `scripts/verify_structure.ps1` icindeki stale `$ExpectedVersion = "0.12.4"` sabiti kaldirildi.
- Header gate exact global release eslesmesi yerine dosya-bazli SemVer `Version:` dogrulamasi yapar.
- `Cargo.toml` gibi v0.21.x dosyalarinin `HEADER_TOKEN_MISSING -> Version: 0.12.4` ile yanlis bloke edilmesi giderildi.
- Windows launcher branding `v0.21.1` oldu ve workspace package version patch bump ile `0.21.1` yapildi.
- Config schema ve Engine API contract degismedi; runtime davranisinda breaking change yoktur.


## v0.21.0 - Transactional Cache + Signed Agent Trust Hardening

- Config schema 16 ve Engine API 17.
- Mutable Artifact Cache gercek URL fetch/import use-case'ine baglandi; ETag/Last-Modified payload ile ayni GET response'dan capture edilir.
- Remote change last-known-good kaydi silmez; replacement basarili store sonrasi atomik olarak promote edilir.
- Source/digest cross-process file lock, PIN preservation ve source-key URL rebinding guard eklendi.
- Cache future schema downgrade durumunda quarantine edilmez; fail-closed `UnsupportedFutureSchema` doner.
- Cache metadata temp/backup/promote/rollback transactional yazima gecti.
- HTTPS/private-network/query/userinfo/redirect policy ve transient-only stale fallback eklendi.
- Redirectler manuel takip edilir ve her Location takip edilmeden once SSRF/network policy'den gecer.
- Desktop mutable URL fetch, remote-modified/LKG semantics ve API v17 wiring'i eklendi.
- Android Guest Agent manifest schema 2 APK SHA-256, Ed25519 ve APK signing-certificate SHA-256 pin'i birlikte dogrular.
- Built-in Agent versionCode 2100 / versionName 0.21.0 olarak explicit tanimlandi.
- AOSP platform.pk8/platform.x509.pem ile update APK sign + Ed25519 manifest ureten release helper eklendi.
- Rollback APK SHA-256 sidecar, installed/candidate cert equality ve rollback hash reverify eklendi.
- TVGB secret/rollback Unix 0600/0700 ve Windows current-user-only ACL hardening'i eklendi; mevcut secret ACL migration uygulanir.
- v0.21 local HTTP contract testi ve Linux/Windows CI gate genisletildi.


## v0.20.0 - Mutable Artifact Revalidation + Desktop Cache Management

- Artifact Cache metadata modeli ETag, Last-Modified ve last-revalidated timestamp alanlariyla schema 2'ye yukseltildi; schema 1 kayitlari backward-compatible okunur.
- Yeni `ArtifactSourceValidationPort` ile mutable HTTP kaynak dogrulamasi cache Service katmanina tasindi.
- Curl tabanli validator `If-None-Match` ve `If-Modified-Since` conditional HEAD kullanir; 304 kaydi korur, degisen 2xx kaydi invalidate eder.
- Validator/network hatasinda config izin veriyorsa daha once SHA-256 dogrulanmis stale cache offline fallback olarak kullanilabilir.
- Android downloader ile Desktop/API yonetimi tek shared `ArtifactCacheService` instance'ini `Arc<Mutex<...>>` uzerinden kullanir; cache mutation race'i engellenir.
- Engine API v16 ile Artifact Cache overview/list/pin/remove/verify/cleanup/revalidate/revalidate-all action ve DTO'lari eklendi.
- Desktop'a Artifact Cache modal paneli eklendi; local ve remote Engine modunda ayni API kullanilir.
- Ana config schema 15'e yukseltildi; mutable revalidation timeout ve offline stale policy merkezi config'e eklendi.
- Linux/Windows CI gate'ine gercek local HTTP 200 -> conditional 304 ETag/Last-Modified contract testi eklendi.
- Desktop `create_vm_disk` tarafinda eski duplicate function-signature compile blocker temizlendi.
- Rust/Cargo toolchain mevcut runtime'da bulunmadigi icin production release gate'i halen gercek Cargo + QEMU runtime testlerini zorunlu tutar.

## v0.19.0 - Artifact Cache + Signed Android Guest Agent Update

- Android CI immutable device/host artifact indirme yolu merkezi Artifact Cache'e baglandi; cache HIT durumunda `curl` yeniden calismaz.
- Cache SHA-256 content-addressed payload, source-index, dedup, verify-on-hit, offline restore, PINNED ve quota/LRU politikalarini uygular.
- Bozuk Artifact Cache source-index kayitlari quarantine edilir; bozuk metadata guvenli MISS olarak yeniden indirmeye duser.
- Android downloader retry, retry-delay ve connect-timeout degerleri merkezi `artifact_cache.downloader` config'ine tasindi.
- Ana config schema 14'e yukseltildi; Engine ve Desktop schema drift durumunda fail-fast davranir.
- Android Guest Agent harici update yolu APK SHA-256 ve Ed25519 signed manifest dogrulamasi ile eklendi.
- Update oncesi mevcut APK rollback icin staging'e alinir; post-install versionCode ve TVGB v2 health-check zorunludur.
- Update sonrasi version/health failure durumunda rollback zorunludur; version downgrade rollback `adb install -r -d` kullanir.
- Rollback isleminin kendisi basarisiz olursa hata yutulmaz ve deployment hatasi acikca raporlanir.
- `AndroidGuestAgentTool::allocate_port()` icindeki yinelenen `PortExhausted` compile blocker temizlendi.
- Linux/Windows Artifact Cache + Agent static regression workflow'u eklendi.
- H-L regression zinciri ile FULL recovery static gate korunur; Cargo ve gercek QEMU guest runtime gate'leri build hostta zorunlu kalir.

## v0.12.4 - Storage + Network Foundation

- Windows Android distribution free-space probe PowerShell `DriveInfo` stdout parsinginden cross-platform `fs2::available_space` API'sine tasindi.
- Core `StorageHostPort` / `StorageHostService` ve storage `HostStorageTool` katmanlari eklendi.
- Engine API v14 `GetStorageOverview`, `CreateVmDisk`, `GetNetworkOverview`, `AttachDefaultNetwork` ve `DetachVmNetwork` actionlarini acar.
- Desktop Storage ve Network nav/modallari aktif edildi; durmus VM icin QCOW2 disk ve default User NAT islemleri Controller -> Service -> Tool akisi ile yonetilir.
- Storage ve Network varsayilanlari merkezi config'e alindi; ana config schema 13'e yukseltildi.
- TurkuazVM portable image uzantisi `.tvmimg`, schema 1 container tipi `ZIP64` olarak tanimlandi; payloadlar standart QCOW2/RAW/Android artifact formatlarinda kalir.
- Runtime policy `private_copy_default: true` oldu. Android runtime shared backing overlay yerine VM'e ozel backing-free `os-private.qcow2` diskini `qemu-img convert` ile uretir.
- `.tvmimg` interoperability ve guvenlik kurallari `docs/TVMIMG_FORMAT.md` altinda tanimlandi.


## v0.12.3 - Android First-Boot Request Timeout Patch

- Desktop host profiline `long_request_timeout_ms` eklendi; varsayilan local ve example profiller 180000 ms kullanir.
- Android `StartVm` first-boot akisi ADB readiness ve opsiyonel Guest Agent provisioning tamamlanana kadar normal 1.5 sn request timeoutuna takilmaz.
- `WaitAndroidReady`, VM stop, snapshot/restore/delete, clone ve APK install gibi uzun sureli operasyonlar ayni typed long-timeout yolunu kullanir.
- Engine API client `send_with_timeout` ile request bazli timeout uygular; normal Dashboard/List gibi hizli cagrilar kisa timeout contractini korur.
- Desktop config long timeout degerini en az 30 sn ve normal request timeoutundan buyuk/esit olacak sekilde fail-fast dogrular.
- Ana config schema 12'ye yukseltildi; Engine API v13 contracti degismedi.

## v0.12.2 - Android Provisioning Operations

- Engine API v13 ile `CancelAndroidImageDistribution` ve `CleanupAndroidImageDistribution` actionlari eklendi.
- Background Android image kurulumu kalici `cancel-requested.flag` marker'i ile kontrollu iptal edilebilir; aktif curl processi yalniz kendi worker'i tarafindan durdurulur.
- FAILED image `Tekrar Dene` ile partial download/resume staging'ini kullanabilir; `Stage Temizle` yalniz READY/INSTALLING olmayan image'larda gecerlidir.
- Install status schema 2 stage baslangic zamani ve byte baseline bilgisini tasir; schema 1 progress dosyalari backward-compatible okunur.
- Desktop progress karti byte/s indirme hizi, ETA ve stage suresini gosterir.
- Local Desktop install logu canonical TurkuazVM `data` root guard'i ile `Logu Ac` aksiyonundan acilabilir; remote path acma engellenir.
- Relative install log yollari Desktop process current-directory'sine degil proje kokune gore cozulur; local log goruntuleme launcher/Tauri cwd farkindan etkilenmez.
- Iptal marker'i bundle atomik aktive edilmeden hemen once tekrar kontrol edilir; finalization sirasindaki iptal commit noktasina kadar korunur.
- Android Images ekranina durmus Android VM secici ve READY image icin tek tik `Secili VM'e Ata` aksiyonu eklendi.
- Basarili kurulumdaki READY immutable backing-image contracti, SHA-256 provenance, retry/resume ve Engine restart recovery davranislari korunur.

## v0.12.1 - Android Auto Provisioning Hardening

- Android CI otomatik kurulumuna minimum bos disk preflight eklendi; varsayilan esik config uzerinden 32 GiB, izin verilen minimum 16 GiB.
- Device ve host paket indirmeleri `curl` retry + partial-file resume ile devam edebilir; CDN resume kabul etmezse partial dosya silinip tek sefer sifirdan retry edilir.
- Background kurulum asamasi `install-status.yml` ile typed progress olarak saklanir ve Engine API v12 uzerinden Desktop'a tasinir.
- Desktop Android Images karti kurulum asamasini, indirilen byte miktarini, varsa yuzde bilgisini ve install log yolunu gosterir.
- Kurulum logu `install.log` dosyasina yazilir; basarili bundle ile birlikte final image klasorune tasinir.
- Launcher `curl.exe` ve Windows `tar.exe` varligini preflight'ta dogrular.
- Ana config schema 11 `android.image.distribution.minimum_free_disk_gib` alanini ekler.
- Ayni build ID icin yarim kalmis device/host download dosyalari yeniden kurulumda korunur; build ID degisirse staging temizlenir. Resmi `aosp_cf_x86_64_only_phone-img-*` adi birinci adaydir.
- Android image READY invariantinda yinelenen `validate_boot_candidate` cagrisi kaldirildi.
- Install progress statusu diagnostic/best-effort okunur; bozuk veya gecici eksik progress dosyasi Android image listesini dusurmez.
- Android CI effective redirect URL kesfi `BUILD_INFO` marker konumuna gore yapilir; trailing query/suffix degisikliklerine karsi daha toleranslidir.

## v0.12.0 - Android Auto Provisioning

- Resmi Android CI `aosp-android-latest-release` / `aosp_cf_x86_64_only_phone-userdebug` latest build kesfi eklendi.
- Device image ile ayni build'e ait `cvd-host_package.tar.gz` background worker uzerinden otomatik indirilir.
- Android Image state modeline `INSTALLING` eklendi; registry schema 2, schema 1 backward read ile korunur.
- Engine API v11 `InstallAndroidImageDistribution` action contractini acar.
- Desktop Android Images ekranina `Otomatik Kur` ve 3 saniyelik INSTALLING durum polling akisi eklendi.
- Device/host archive SHA-256 provenance `distribution.yml` ile bundle icinde kaydedilir.
- Archive absolute/parent path preflight ve staging + backup rollback activation akisi eklendi.
- Cuttlefish host paketinden x86_64 `bootloader.qemu` otomatik bulunur.
- Android sparse image formatini dogrudan okuyabilen Rust composite GPT assembler eklendi.
- Stok Google Cuttlefish image capability'si Guest Agent ve persistent multi-touch olmadan kaydedilir; ilk boot ADB/display readiness ile ilerleyebilir.
- Android image YAML repository background worker ile UI polling arasinda ortak mutex ile serialize edilir; manifest commit temp/backup/rename akisi ile korunur.
- READY Android image bundle immutable tutulur; otomatik reinstall ayni Image ID uzerinde engellenerek mevcut QCOW2 backing contracti korunur.
- Engine restart sirasinda yarida kalmis INSTALLING kayitlari FAILED durumuna toparlanir ve yeniden denenebilir.
- Ana config schema 10 Android CI branch/target/curl ayarlarini tasir.
- Android Gaming Renderer kilometre tasi v0.13.0'a kaydirildi.


## v0.11.9 - Build Warning Cleanup Patch

- Windows gercek build testinde gorulen 22 Rust `dead_code` warningi kaynak seviyesinde temizlendi.
- EngineConfigError, DesktopConfigError ve EngineApiClientError mesaj alanlari `Display`/`Error` contracti ile gercekten kullanilir hale getirildi; kullaniciya hata metni Debug yerine Display ile aktarilir.
- Aktif olmayan eski EngineController, GuestBootController, NetworkController, StorageController ve VmController scaffold dosyalari kaldirildi; tek aktif request girisi EngineApiController -> EngineApplicationService olarak netlestirildi.
- Kullanilmayan Engine API `peer_addr`, Android image application `get()` facade'i ve ConsoleView host capability renderer kaldirildi.
- Workspace Rust lint contractina `dead_code = deny` eklendi; ayni sinif warning tekrar build'e girerse compile kapisi release'i durdurur.
- Mimari dokumanlar aktif Controller -> Service akisi ile hizalandi.
- v0.11.8 acik tema ve onceki Windows/QEMU/Tauri patchleri korunur.

## v0.11.8 - Light Desktop Theme Patch

- TurkuazVM Desktop varsayilan arayuzu acik temaya gecirildi.
- Sidebar, status kartlari, VM kartlari, modal, input ve secim alanlari acik zemin kontrastina gore yeniden dengelendi.
- Turkuaz vurgu rengi korunurken ana metin koyu, ikincil metin gri-yesil ve hata renkleri acik zeminde okunur hale getirildi.
- Modal backdrop daha hafif blur ve saydamlik ile acik tema davranisina uyarlandi.
- CSS `color-scheme: light` contracti ve temel acik tema tokenlari structure verifier kapsaminda dogrulanir.
- v0.11.7 Tauri icon, v0.11.6 Softbuffer ve onceki Windows launcher patchleri aynen korunur.

## v0.11.7 - Tauri Windows Icon Build Patch

- Tauri Windows resource build icin zorunlu `apps/desktop/src-tauri/icons/icon.ico` asseti eklendi.
- ICO 16, 24, 32, 48, 64, 128 ve 256 px varyantlarini tek Windows icon resource icinde tasir.
- `tauri.conf.json5` bundle icon yolu `icons/icon.ico` olarak acikca tanimlandi.
- Structure verifier icon dosyasinin varligini, Tauri config referansini ve temel ICO header contractini dogrular.
- v0.11.6 Softbuffer 0.4.8 compile patch'i aynen korunur.

## v0.11.6 - Softbuffer 0.4.8 Display Compile Patch

- TurkuazDisplay native renderer `softbuffer 0.4.8` API contractina guncellendi.
- Eski `Pixel` importu kaldirildi.
- Eski `Surface::next_buffer()` cagrisi `Surface::buffer_mut()` ile degistirildi.
- Eski `pixels_iter()` akisi yerine Softbuffer `Buffer` uzerinde row-major `u32` pixel yazimi kullanilir.
- Framebuffer pixel degeri Softbuffer 0x00RRGGBB contractina uygun olarak 24-bit RGB maskesiyle yazilir.
- v0.11.5 preflight, structure verifier, WHPX probe ve Windows dependency bootstrap davranislari korunur.

## v0.11.5 - Desktop Game Profile Verifier Patch

- Game profile statik DOM contractlari `apps/desktop/ui/index.html` uzerinden dogrulanmaya devam eder.
- JavaScript tarafinda dinamik uretilen `data-game-action` contracti artik `apps/desktop/ui/app.js` uzerinden dogrulanir.
- `compatibility` ve `apply` action butonlari ile delegated click selector ayri verifier kapisina alindi.
- Gecerli Desktop UI kodunun yanlis `GAME_PROFILE_DESKTOP_VIEW_MISSING: data-game-action` hatasi ile launcher'i durdurmasi engellendi.
- v0.11.4 Windows path normalization ve v0.11.3 WHPX runtime probe davranislari korunur.

## v0.11.4 - Windows Header Path Verifier Patch

- Windows dosya yollarindaki `\` ayirici `/` bicimine merkezi olarak normalize edilir.
- `.gitignore` dahil kok dosyalarinin header path dogrulamasi Windows PowerShell 5.1 ile uyumlu hale getirildi.
- Verifier basinda `/.gitignore` path normalization regression probe eklendi.
- WHPX runtime probe ve v0.11.3 sanallastirma karari aynen korunur.

## v0.11.3 - WHPX Runtime Probe Patch

- Launcher firmware virtualization WMI sonucunu tek basina bloke edici olmaktan cikardi.
- QEMU ile gercek WHPX runtime smoke testi eklendi.
- Win32_ComputerSystem HypervisorPresent bilgisi ayri tanisal satir olarak eklendi.
- WMI DISABLED raporlasa bile WHPX probe basariliysa baslatma devam eder.
- Gercek WHPX init hatasi probe stderr ayrintisi ile launcher loguna yazilir.

## 0.11.2 - Windows Dependency Bootstrap

- Windows launcher eksik QEMU, ADB ve WebView2 bagimliliklarini WinGet ile kullanici onayi sonrasinda kurabilir.
- QEMU package kimligi `SoftwareFreedomConservancy.QEMU`, ADB `Google.PlatformTools`, WebView2 `Microsoft.EdgeWebView2Runtime` olarak merkezi sabitlere alindi.
- QEMU ve WinGet portable arac dizinleri kurulumdan sonra mevcut launcher prosesine yeniden eklenir.
- WebView2 Runtime registry detection Microsoft resmi `{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` product key ve `pv` version kontrolune duzeltildi.
- Windows Hypervisor Platform durumu WMI ile okunur; kapaliysa kullanici onayi ile elevated DISM activation akisi sunulur.
- Firmware virtualization durumu preflight ekranina eklendi.
- Python composite builder dosyasi header verifier kapsaminda teknik dosya olarak dogrulanir.
- Launcher bootstrap contractlari structure verifier icine eklendi.
- Cuttlefish blank metadata fallback boyutu referans varsayilan deger olan 16 MiB ile hizalandi.

## 0.11.1 - Windows Test Launcher

- Proje kokune tek tik Windows `TurkuazVM-Start.cmd` launcher eklendi.
- Launcher is kurallarini `scripts/start_turkuazvm.ps1` Tool katmaninda tutar.
- Rust/Cargo, QEMU ve WHPX preflight kontrolleri eklendi.
- Cargo ve QEMU icin standart Windows kurulum yollarini PATH'e otomatik ekleme eklendi.
- Visual Studio Build Tools gelistirme ortamini `vswhere` + `VsDevCmd` ile normal terminalden otomatik yukleme eklendi.
- Android SDK `adb` yolu opsiyonel olarak otomatik bulunur; Android ilk boot testi icin eksikse acik uyari verilir.
- Launcher calisma loglari `data/logs/launcher` altina yazilir.
- Native komut exit-code kontrolu fail-fast hale getirildi.
- Structure verifier launcher dosyalarini ve `.cmd` header sozlesmesini kapsayacak sekilde genisletildi.


## 0.11.0 - Android Boot Integration

- Core `VmRuntimeMediaPlan` ile guest-specific runtime boot media contracti eklendi.
- Cuttlefish partition artifactlerini A/B GPT `composite.img` icinde birlestiren Python builder eklendi.
- `bootloader.qemu` ve `composite.img` READY image invariantina eklendi.
- Registered composite image ortak immutable template olarak tutulur; her VM ve image eslesmesi icin QCOW2 overlay olusturulur.
- Cuttlefish U-Boot firmware icin readonly bootloader pflash ve VM ve image-ozel writable pflash state eklendi.
- QEMU Android runtime media launch spec generic VM disklerinden ayrildi.
- Android image assignment schema version 2, boot attempt sayaci ve schema 1 backward read eklendi.
- Android ilk Start akisi ADB readiness, display apply ve Guest Agent provisioning ile baglandi.
- TurkuazInputAgent AccessibilityService mevcut enabled service listesini koruyarak ADB ile etkinlestirilir.
- Guest Agent first-boot readiness gercek Ping/Pong handshake ile dogrulanir.
- First-boot hatasinda assignment `Failed` olur ve VM kontrollu stop edilir; basarida `Ready` olur.
- Engine API version 10 assignment boot attempt ve yeni artifact rollerini acar.
- Config schema version 9 Guest Agent readiness timeout/poll alanlarini zorunlu hale getirir.
- Desktop Android assignment gorunumu boot attempt bilgisini gosterir.
- Engine ve Desktop bundle surum kimlikleri 0.11.0 ile hizalandi.


## 0.10.0 - Turkuaz Android Image Foundation

- Yeni `turkuazvm-android-image` bounded context crate'i eklendi.
- Android image build profile, artifact, capability, state ve assignment domain modelleri eklendi.
- Image assignment provisioning state `pending_first_boot` ile baslar; first-boot readiness v0.11 lifecycle tarafindan tamamlanacaktir.
- `AndroidImageRepositoryPort` ve `AndroidImageBuilderPort` eklendi.
- `YamlAndroidImageRepository` image registry ve VM assignment persistence'i eklendi.
- Android image manifest schema version 1 eklendi.
- `AospAndroidImageTool` build plan ve checksum'lu bundle registration adapteri eklendi.
- SHA-256 artifact fingerprint ve READY artifact invariantlari eklendi.
- AOSP `android-latest-release` source preparation scripti eklendi.
- Linux 400 GiB minimum free disk preflight eklendi.
- Cuttlefish x86_64-only Turkuaz Android product overlay eklendi.
- `TurkuazInputAgent` AOSP product image paketine dahil edildi.
- External AOSP build scripti boot/super/userdata ve ek artifact bundle'i uretir.
- Engine config schema 8 icine `android.image` source/output/build-script ayarlari eklendi.
- `AndroidImageApplicationService` Engine composition'ina eklendi.
- Engine API version 9 image list/define/build-plan/register/assign/get-assignment contractlari ile eklendi.
- Desktop'a Android Images manager, build plan gorunumu ve Android VM image assignment UI eklendi.
- AOSP source ve uzun build islemi Engine API request thread'inden ayri tutuldu.
- ARM translation capability v0.10.0'da false olarak tutuldu.
- QEMU Android partition boot entegrasyonu v0.11.0 kapsaminda birakildi.

## 0.9.0 - Game Profiles & Compatibility

- Yeni `turkuazvm-game-catalog` bounded context crate'i eklendi.
- Game Catalog YAML repository ve schema version 1 eklendi.
- PUBG MOBILE icin regional package eslestirmeli deneysel katalog profili eklendi.
- Catalog key/mouse kaynaklari typed enum yapisina alindi; Engine Service string parsing temizlendi.
- Duplicate game ID/package ve pointer/binding invariantlari fail-fast hale getirildi.
- Engine API version 8 Game Catalog, detection, compatibility, profile apply ve Guest Agent status contractlari ile eklendi.
- `turkuazvm-guest-agent-protocol` protocol version 1 eklendi.
- ADB forward tabanli `AndroidGuestAgentTool` eklendi.
- Guest Agent stale forward cleanup ve Engine shutdown cleanup eklendi.
- Referans AOSP `TurkuazInputAgent` AccessibilityService uygulamasi eklendi.
- Persistent multi-touch `TouchFrame` artik agent capability handshake READY ise uygulanir.
- Android API 26+ stroke continuation referans implementasyonu eklendi.
- TurkuazDisplay icin concrete GilRs Windows/Linux gamepad adapteri eklendi.
- Gamepad button/axis degerleri TurkuazVM normalize ID'lerine cevrildi.
- Desktop Android Runtime paneline Guest Agent, Game Catalog, installed-game detection, compatibility ve profile apply UI eklendi.
- Game profile apply Android runtime hatasinda Gaming Input profile rollback uygular.
- Anti-cheat/emulator identity bypass kapsam disi olarak korunur.

## 0.8.0 - Gaming Input Foundation

- Yeni `turkuazvm-gaming-input` bounded context crate'i eklendi.
- Normalized 0-10000 koordinat modeli ile cozunurlukten bagimsiz keymapping profili eklendi.
- WASD virtual joystick ve relative mouse-look icin VM bazli stateful translator eklendi.
- Keyboard/mouse source -> Tap, HoldTouch ve AndroidKey typed binding modeli eklendi.
- Joystick, mouse-look ve HoldTouch pointer ID cakismalari domain seviyesinde engellendi.
- `gaming-input.yml` schema version 1 ve YAML repository adapteri eklendi.
- Engine API version 7 Gaming Input capability/profile/event/reset contractlari ile eklendi.
- TurkuazDisplay raw keyboard ve `DeviceEvent::MouseMotion` eventlerini bounded local bridge ile Engine'e aktarir.
- F1 capture state Gaming Input translator'a MouseCapture eventi olarak iletilir.
- ADB single-touch Tap ve AndroidKey fallback'i Engine tarafinda uygulanir.
- Persistent multi-touch TouchFrame planlari guest-agent gelene kadar acik `gaming_input_guest_agent_required` hatasi dondurur.
- `HostGamepadPort` ve gamepad domain eventleri eklendi; concrete host gamepad capability false kalir.
- Desktop Android Runtime icine Gaming Input / Keymapping Editor eklendi.
- Local Engine token child process command-line yerine environment ile TurkuazDisplay'e aktarilir.
- Remote/TLS native Gaming Input bridge v0.8.0 kapsaminda bilincli olarak devre disidir.

## 0.7.0 - Gaming GPU Foundation

- Yeni `turkuazvm-gpu` bounded context crate'i eklendi.
- HostGpuProbePort ve HypervisorGpuProbePort contractlari eklendi.
- NativeGpuProbeTool ile Windows/Linux Vulkan/OpenGL runtime discovery eklendi.
- QemuGpuProbeTool ile virtio-gpu, VirGL, Venus, rutabaga ve GfxStream property discovery eklendi.
- GamingGpuPolicy ve safe `auto` backend resolution eklendi.
- Experimental Android GfxStream explicit opt-in olmadan secilmez.
- Stock QEMU accelerated backend resolution Linux host ile sinirlandi; Windows yanlis pozitif acceleration engellendi.
- QemuGpuRuntimeSettings ve GPU-aware QemuCommandBuilder eklendi.
- `virtio-gpu`, `virtio-gpu-gl + blob + venus` ve experimental `virtio-gpu-rutabaga` launch stratejileri eklendi.
- VirGL/Venus + native RFB durumunda `egl-headless,gl=on` display context strategy eklendi.
- Engine API version 6 `GetGpuCapabilities` contracti ile eklendi.
- Dashboard GPU backend/Vulkan/acceleration bilgisi ile genisletildi.
- Desktop Gaming GPU capability modal ve status card eklendi.
- Config'e `gpu.mode`, `hostmem_mib` ve experimental opt-in eklendi.
- TurkuazDisplay telemetry FPS yanina average frametime ms ekledi.
- QEMU builder GPU unit testleri eklendi.

## 0.6.0 - Android Runtime Foundation

- `turkuazvm-android` bounded context crate'i eklendi.
- `GuestProfile::Android` ve Create VM guest profile secimi eklendi.
- Engine API version 5 Android runtime/package/input contractlari ile eklendi.
- Android runtime profili `machines/<vm-id>/android.yml` schema version 1 ile ayrildi.
- ADB binary discovery explicit config, Android SDK environment ve PATH sirasi ile eklendi.
- ADB capability, device state, boot-completed, SDK, ABI ve model inspection eklendi.
- Android VM configure use-case'i bos loopback ADB host portu ayiracak sekilde eklendi.
- Dedicated `android-nat` TCP host-forward -> guest 5555 provisioningi eklendi.
- Network provisioning failure compensation icin typed `DetachNetworkCommand` eklendi.
- Mevcut Android network/profile port uyumsuzlugu fail-fast guard ile engellendi.
- Android display width/height/density/target FPS profile ve apply use-case'i eklendi.
- APK relative path ve canonical package-root containment guvenlik siniri eklendi.
- APK install/uninstall, package list, launch ve force-stop eklendi.
- Tap, swipe, keyevent ve text input injection port/service/ADB adapteri eklendi.
- Desktop Android Runtime modal, device status, package manager ve basic input test UI eklendi.
- Dashboard ADB readiness ve version bilgisi ile genisletildi.
- AOSP image build, ARM ABI translation ve Gaming GPU bilincli olarak sonraki kilometre taslarina birakildi.

## 0.5.0 - Native Display ve Input

- Core display domain modelleri ve protocol-neutral `DisplayRuntimeInfo` eklendi.
- Hypervisor runtime contracti display session bilgisi ile genisletildi.
- QEMU runtime icin `NativeRfb`, `Default` ve `None` display stratejileri ayrildi.
- Loopback RFB display port allocation eklendi.
- QEMU `-vnc 127.0.0.1:N,share=force-shared` launch adapteri eklendi.
- RFB bind v0.5.0 icin kesin olarak `127.0.0.1` ile sinirlandi.
- Engine API version 4 `GetDisplaySession` contracti ile eklendi.
- Ayri `turkuazvm-display` native process crate'i eklendi.
- RFB 3.3/3.7/3.8 version negotiation ve None-auth loopback guard eklendi.
- Raw framebuffer ve DesktopSize decoding eklendi.
- Bounded frame queue ile eski frame biriktirmeme politikasi eklendi.
- Winit native window ve Softbuffer CPU fallback renderer eklendi.
- F1 mouse capture, Escape release ve F11 fullscreen eklendi.
- Winit raw `DeviceEvent::MouseMotion` capture eklendi.
- Physical keyboard code -> RFB keysym adapteri eklendi.
- FrameClockTool FPS/present timing foundation eklendi.
- Desktop VM kartina `Ekran` aksiyonu eklendi.
- Desktop display process tool ve local-only remote guard eklendi.
- Gamepad capability gercek adapter gelene kadar `false` olarak tutuldu.
- `run_desktop.ps1` Engine ile birlikte TurkuazDisplay binary'sini build edecek sekilde guncellendi.

## 0.4.0 - Remote Host

- Engine API version 3 auth token contracti ile eklendi.
- `turkuazvm-transport` crate ve rustls TLS stream adapteri eklendi.
- Loopback disi Engine bind icin TLS + token zorunlulugu eklendi.
- QMP loopback zorunlulugu korundu.
- EngineAuthService ve token validation eklendi.
- Token secret file ve `TURKUAZVM_ENGINE_TOKEN` destegi eklendi.
- Engine host identity ve transport security bilgisi Dashboard'a eklendi.
- Desktop local/remote host profile modeli eklendi.
- Desktop host selector ve runtime host switching eklendi.
- Remote profilde TLS CA/server-name validation eklendi.
- Local Engine auto-start yalniz local profile icin korundu.
- Linux headless Engine remote config ornegi eklendi.
- Windows Desktop local + remote config ornegi eklendi.
- Remote credential bootstrap scripti ve systemd deployment sablonu eklendi.
- `secrets/` Git ignore kapsamina alindi.

## 0.3.0 - Snapshot ve Clone

- SnapshotId ve SnapshotRecord domain modelleri eklendi.
- SnapshotPort ve SnapshotService eklendi.
- Offline QCOW2 snapshot create/list/restore/delete akislari eklendi.
- Multi-disk snapshot create rollback ve partial restore/delete typed hata modelleri eklendi.
- CloneMode, CloneStoragePort ve CloneService eklendi.
- Full clone `qemu-img convert` ile eklendi.
- Linked clone target-local immutable QCOW2 base ve aktif overlay zinciri ile eklendi.
- Clone guest media ve firmware asset copy eklendi.
- Clone transaction target ownership ve cleanup compensation eklendi.
- Klon VM network adapterleri duplicate MAC/host bind riskine karsi sifirlanir.
- machine.yml schema version 5 snapshot metadata persistence eklendi.
- Schema version 1-4 manifestleri icin bos snapshot geriye uyumu korundu.
- Engine API version 2 snapshot/clone contractlari ile eklendi.
- QEMU ve qemu-img readiness ayrimi dashboard'a eklendi.
- Desktop Snapshot Manager ve Clone Wizard eklendi.

## 0.2.0 - Desktop Foundation

- Tauri 2 vanilla Desktop shell eklendi.
- Versioned `turkuazvm-engine-api` contract crate eklendi.
- Local JSON line Engine API server/client transportu eklendi.
- Desktop Engine auto-start ve startup retry akisi eklendi.
- SharedVmRepository ile query ve lifecycle servisleri ayni persistence instance'ini paylasiyor.
- VmRepositoryPort `list` use-case'i ve VmQueryService eklendi.
- Dashboard, VM list, create, start ve stop Tauri commandlari eklendi.
- Runtime dashboard snapshotlari Tauri event ile UI'ya aktariliyor.
- Dark/turkuaz VM manager View ve basic create wizard eklendi.

## 0.1.6 - Platform Networking

- BridgeConfiguration ve HostNetworkName domain modelleri eklendi.
- NetworkRuntimePlan ve PreparedNetworkBackend modelleri eklendi.
- NetworkPort runtime prepare/bind/cleanup/recovery contractlari ile genisletildi.
- NativeNetworkTool Windows/Linux platform adapteri eklendi.
- Windows existing TAP adapter preflight eklendi.
- Linux existing TAP ve bridge preflight eklendi.
- Linux bridge helper strategy eklendi.
- Linux managed TAP create/bridge/up/delete fallback eklendi.
- Runtime network lease YAML persistence eklendi.
- Lease kaydina QEMU process PID binding eklendi.
- Crash recovery calisan QEMU PID'lerini koruyacak sekilde eklendi.
- VM lifecycle start/stop akisi network prepare ve cleanup ile birlestirildi.
- Start/bind failure compensation ile host network cleanup eklendi.
- QemuCommandBuilder prepared user NAT, host TAP ve host bridge backendlerini destekleyecek sekilde genisletildi.
- QemuNetworkTool kaldirildi; host network karari platform adapterine tasindi.
- machine.yml schema version 4 bridge target persistence eklendi.
- NetworkPrepare, NetworkBind ve NetworkCleanup failure turleri eklendi.

## 0.1.5 - Guest Boot Foundation

- GuestProfile, BootDevice ve BootOrder domain modelleri eklendi.
- GuestBootConfiguration aggregate parcasi eklendi.
- BIOS ve UEFI FirmwareSelection ayrildi.
- MediaId ve IsoAttachment modelleri eklendi.
- PrepareGuestBootCommand ve InstallerIsoCommand eklendi.
- GuestBootService eklendi.
- MediaPort ve FirmwarePort contractlari eklendi.
- turkuazvm-guest crate eklendi.
- LocalGuestMediaTool ile ISO import ve rollback eklendi.
- UefiFirmwareTool ile global code ve VM-local writable vars preparation eklendi.
- GuestBootController eklendi.
- VirtualMachine aggregate guest boot configuration ile genisletildi.
- machine.yml schema version 3 guest persistence eklendi.
- schema version 1/2 manifestleri icin default BIOS/disk boot geriye uyumu eklendi.
- QemuCommandBuilder UEFI pflash, CD-ROM ve boot order argumanlari ile genisletildi.
- UEFI code/vars ve installer ISO preflight kontrolleri eklendi.
- QemuDisplayMode::Default eklendi; guest installer icin visible QEMU display yolu acildi.

## 0.1.4 - Network Foundation

- NetworkId, NetworkAttachment ve typed network domain modelleri eklendi.
- User NAT ve Bridge modlari domain seviyesinde ayrildi.
- TCP/UDP PortForwardRule ve host bind cakisma validasyonu eklendi.
- MAC address parser ve validation eklendi.
- AttachNetworkCommand ve NetworkService eklendi.
- NetworkPort ve NetworkCapabilities contractlari eklendi.
- QemuNetworkTool User NAT adapteri eklendi.
- Bridge capability v0.1.4 icin bilincli olarak kapali tutuldu.
- QemuCommandBuilder -netdev user ve hostfwd argumanlari ile genisletildi.
- virtio-net-pci ve e1000 device modelleri eklendi.
- machine.yml schema version 2 network persistence eklendi.
- schema version 1 manifestleri icin geriye uyumlu okuma eklendi.
- NetworkController eklendi.

## 0.1.3 - Storage Foundation

- DiskId, DiskImage ve DiskAttachment domain modelleri eklendi.
- DiskFormat ve DiskBus enumlari eklendi.
- Relative disk path guvenlik validasyonu eklendi.
- StoragePort eklendi.
- StorageService eklendi.
- CreateDiskCommand ve ResizeDiskCommand eklendi.
- Disk create/attach sadece Stopped VM icin izinli hale getirildi.
- Disk shrink reddedildi; grow-only resize politikasi eklendi.
- turkuazvm-storage crate eklendi.
- QemuImgTool ile qemu-img create/info/resize/delete eklendi.
- qemu-img JSON info parse destegi eklendi.
- YamlVmRepository eklendi.
- machine.yml schema version 1 eklendi.
- VM kaynak, state, failure ve disk attachment persistence eklendi.
- QEMU command builder disk attachment argumanlari ile genisletildi.
- QemuRuntimeSettings icine data_root eklendi.
- StorageController eklendi.
- config/turkuazvm.yml merkezi runtime/storage config dosyasi eklendi.

## 0.1.2 - VM Lifecycle

- VirtualMachine aggregate eklendi.
- VmId ve VmResourceConfig domain modelleri eklendi.
- VmState state machine eklendi.
- CreateVmCommand, StartVmCommand ve StopVmCommand eklendi.
- VmLifecycleService eklendi.
- VmRepositoryPort eklendi.
- HypervisorRuntimePort eklendi.
- InMemoryVmRepository adapter crate eklendi.
- QemuCommandBuilder eklendi.
- QemuRuntimeTool process registry eklendi.
- QMP session QEMU process lifecycle ile birlestirildi.
- QMP quit graceful process termination eklendi.
- Shutdown timeout sonrasi process kill fallback eklendi.
- Hypervisor start/stop failure durumunda VM Error state kaydi eklendi.
- Running state persistence failure durumunda runtime stop compensation eklendi.

## 0.1.1 - QMP Core

- HypervisorMonitorPort eklendi.
- HypervisorMonitorService eklendi.
- QMP greeting modeli eklendi.
- qmp_capabilities handshake eklendi.
- query-version introspection eklendi.
- query-commands introspection eklendi.
- Request/response id correlation eklendi.
- Asynchronous event ayiklama eklendi.
- QMP TCP transport portu ve adapteri eklendi.
- Fake transport unit testleri eklendi.

## 0.1.0 - Core Bootstrap

- Cargo workspace olusturuldu.
- Core, QEMU, platform ve engine sinirlari kuruldu.
- Host capability detection eklendi.
- QEMU discovery eklendi.

## v0.21.9 - Desktop Navigation Behavior Fix

- Sol navigasyon aktif durum yonetimi duzeltildi.
- VM agaci statik orneklerden gercek dashboard.machines verisine tasindi.
- VM agacindan makine secme ve filtreleme eklendi.
- Ctrl+K VM arama ve filtre temizleme davranisi eklendi.
- Islevsiz gorunen dekoratif kontroller arayuzden kaldirildi.


## v0.25.0 - Installer Media Download Center

- Kurulum Medyasi modalina resmi ISO katalog paneli eklendi.
- Guest Catalog installer_media metadata ile direct ve official_page kaynaklarini tanimlar.
- Ubuntu direct ISO indirmeleri background worker ile yapilir.
- Direct medyada SHA-256 hesaplama ve saglayici SHA256SUMS dogrulamasi eklendi.
- Indirme durumu, byte ilerlemesi ve tek tikla indirilen ISO baglama eklendi.
- Microsoft ve Debian icin resmi indirme sayfasi akisi eklendi.
- Yerel ISO secme ve baglama fallback olarak korunur.
- Engine API v20, config schema 19 ve Guest Catalog schema 2'ye yukseltildi.
