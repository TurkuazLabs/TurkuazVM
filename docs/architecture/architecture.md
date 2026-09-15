# 📄 Dosya Yolu: /turkuazvm/docs/architecture/architecture.md
# 📌 Amac: TurkuazVM guncel mimari kurallarini ve dependency yonunu kilitler
# 📌 Modul - Markdown
# Version: 0.32.0
# Aciklama: Modular Monolith, Clean Architecture, Ports & Adapters, virtualization, remote host, native display/input, Android Runtime ve Gaming GPU tasarimini tanimlar
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# TurkuazVM Architecture v0.32.0

## Mimari

TurkuazVM, Modular Monolith + Clean Architecture + Hexagonal Ports & Adapters modeli ile gelistirilir.

## Dependency yonu

Desktop/CLI -> Engine Controller -> Application Service -> Domain/Ports <- Adapters

View yalnizca domain/application sonucunu kullaniciya sunar; controller output formatlamaz.

Core crate; QEMU, QMP JSON, qemu-img, serde/YAML, Tauri, Windows API veya Linux API implementasyonu import edemez.

## Ana moduller

- core: Command, Domain, Port ve Service
- qemu: QEMU/QMP runtime, boot spec ve prepared network plan rendering
- guest: ISO media ve UEFI firmware preparation adapterleri
- storage: qemu-img disk adapterleri
- platform: Windows/Linux host, portable QEMU NAT planlama, Advanced TAP/bridge preflight, runtime lease ve recovery adapterleri
- repositories: Memory/YAML persistence adapterleri
- engine-api: Desktop/Engine versioned transport contractlari
- engine: Local API composition root, Controller, Service ve transport tools
- desktop: Tauri Controller, DesktopService, Engine client tools ve management View
- display: Winit native window, RFB client, input capture ve fallback renderer process
- android: Android runtime/package/input domain, ports ve services
- android-image: AOSP build profile, artifact registry, image state ve assignment domain/ports/services
- gpu: Gaming GPU capability, policy, backend preference ve resolution services

## Host capability vertical slice

1. NativeHostProbeTool host platform ve mimarisini okur.
2. QemuDiscoveryTool QEMU system ve qemu-img binary bilgilerini bulur.
3. HostCapabilityService sonuclari birlestirir.
4. Engine composition root adapterleri service'e enjekte eder.

## QMP vertical slice

1. HypervisorMonitorPort core tarafinda protocol-neutral contract saglar.
2. HypervisorMonitorService runtime kontrol kanali use-case'ini yonetir.
3. QmpClientTool greeting mesaji ile session baslatir.
4. QmpClientTool qmp_capabilities handshake komutunu gonderir.
5. query-version ve query-commands ile runtime introspection yapilir.
6. QMP response id degeri request id ile eslestirilir.
7. Asynchronous event mesajlari response olarak yorumlanmaz.
8. QmpTcpTransportTool cross-platform TCP transport saglar.
9. QMP wire JSON modelleri sadece qemu crate icinde bulunur.
10. quit komutu process termination icin QEMU runtime tarafinda kullanilir.

## VM lifecycle vertical slice

1. EngineApiController request'i alir ve EngineApplicationService'e aktarir; application service lifecycle use-case'ini VmLifecycleService'e delege eder.
2. VmLifecycleService VirtualMachine aggregate state kurallarini uygular.
3. VmRepositoryPort persistence sinirini belirler.
4. HypervisorRuntimePort process lifecycle sinirini belirler.
5. QemuRuntimeTool QEMU process ve QMP session'i birlikte yonetir.
6. QemuCommandBuilder domain modelini QEMU launch argumanlarina cevirir.
7. Runtime failure VM aggregate icinde Error state ve typed failure ile temsil edilir.

## Storage vertical slice

1. EngineApiController request'i EngineApplicationService'e aktarir; v0.12.4 Engine API v14 Storage overview ve disk create use-case'lerini acar.
2. StorageService VM'nin Stopped oldugunu dogrular.
3. StoragePort fiziksel image islemlerini abstract eder.
4. QemuImgTool qemu-img create/info/resize/delete komutlarini uygular.
5. VirtualMachine disk attachmentlari domain icinde tutulur.
6. YamlVmRepository aggregate bilgisini machine.yml manifestine yazar.
7. QemuCommandBuilder attached diskleri runtime launch spec'e ekler.


## Network vertical slice

1. EngineApiController request'i EngineApplicationService'e aktarir; Engine API v24 Network overview, profil attach, service publish ve detach use-case'lerini acar.
2. NetworkService VM'nin Stopped oldugunu dogrular ve typed network domain kurallarini uygular.
3. NetworkPort capability, prepare, PID bind, cleanup ve recovery sinirini belirler.
4. `managed_nat` Windows/Linux icin NativeNetworkTool tarafindan `PreparedNetworkBackend::UserNat` olarak hazirlanir; host TAP kaynagi gerektirmez.
5. `private` ve `bridge` Advanced Network profilleri platform-specific TAP/bridge adapterlerini kullanabilir.
6. VirtualMachine network attachment, bridge target ve cross-adapter host bind cakismalarini korur.
7. VmLifecycleService network runtime'i QEMU start/stop ile transaction benzeri compensation akisi icinde orkestre eder.
8. YamlVmRepository network bilgisini machine.yml schema version 6 icinde saklar.
9. QemuCommandBuilder managed IPAM bilgisini QEMU `net/host/dhcpstart`, servis yayinlarini `hostfwd` ve diger prepared backendleri TAP/bridge argumanlarina cevirir.
10. Runtime lease QEMU PID bilgisini saklar; v0.31.x stale ManagedTap lease'i portable NAT startini bloklamaz.

## Guest boot vertical slice

1. EngineApiController request'i EngineApplicationService'e aktarir; guest boot use-case API'ye acildiginda application service GuestBootService sinirini kullanir.
2. GuestBootService sadece Stopped VM icin boot configuration degisikligine izin verir.
3. MediaPort installer ISO import/rollback sinirini belirler.
4. FirmwarePort UEFI preparation/cleanup sinirini belirler.
5. LocalGuestMediaTool ISO'yu VM-local media klasorune kopyalar.
6. UefiFirmwareTool ortak read-only code ve VM-local writable vars dosyalarini hazirlar.
7. YamlVmRepository guest boot bilgisini guncel machine.yml schema version 5 icinde saklar.
8. QemuCommandBuilder BIOS/UEFI, CD-ROM ve boot order bilgisini launch spec'e cevirir.
9. QemuDisplayMode::Default installer penceresi icin host display backend'ini acik birakir.


## Snapshot ve clone vertical slice

1. SnapshotService yalniz Stopped VM ve QCOW2 disk seti icin offline snapshot islemlerini orkestre eder.
2. SnapshotPort qemu-img internal snapshot detaylarini Core'dan ayirir.
3. Tum disklerde ayni SnapshotId kullanilir; create hatasinda tamamlanan diskler rollback edilir.
4. Snapshot metadata VirtualMachine aggregate icinde tutulur ve machine.yml schema version 5 ile saklanir.
5. CloneService Full ve Linked clone transaction akislarini yonetir.
6. Full clone bagimsiz disk image uretir.
7. Linked clone hedef VM icinde immutable local base image ve onun uzerinde aktif QCOW2 overlay olusturur.
8. Clone hedefi network attachment ve snapshot metadata miras almaz; MAC ve host port cakismalari engellenir.
9. Clone aggregate veya persistence adimi basarisiz olursa transaction-owned hedef storage temizlenir.
10. Engine API v2 snapshot ve clone use-case'lerini Desktop'a versioned DTO ile acar.

## Desktop vertical slice

1. Vanilla View yalniz Tauri command ve runtime event contractlarini kullanir.
2. DesktopController invoke requestini DesktopService'e aktarir.
3. DesktopService Engine API requestini olusturur ve gerekiyorsa Engine process auto-start politikasini uygular.
4. EngineApiClientTool loopback TCP JSON line transportu uygular.
5. EngineApiServerTool request I/O detaylarini Engine Controller'dan ayirir.
6. EngineApiController request'i EngineApplicationService'e aktarir.
7. EngineApplicationService HostCapabilityService, VmQueryService ve VmLifecycleService use-case'lerini orkestre eder.
8. SharedVmRepository query ve lifecycle service'lerinin ayni persistence instance'ini kullanmasini saglar.
9. Desktop runtime update composition root tarafindan kucuk dashboard snapshotlari olarak Tauri event kanalina aktarilir.


## Native display ve input vertical slice

1. QemuRuntimeTool local native display icin bos loopback RFB display number ayirir.
2. QemuCommandBuilder display planini `-display none` ve loopback VNC/RFB launch argumanina cevirir.
3. HypervisorRuntimeInfo protocol-neutral DisplayRuntimeInfo bilgisini Core Service'e dondurur.
4. Engine API v4 ile eklenen `GetDisplaySession` typed display endpoint/capability DTO'su Engine API v7 icinde korunur.
5. DesktopService yalniz local host profilinde display session ister.
6. DisplayProcessTool `turkuazvm-display` process'ini baslatir.
7. DisplayController yalniz DisplayService'i cagirir.
8. RfbClientTool RFB handshake, framebuffer ve input wire protokolunu uygular.
9. NativeDisplayView Winit event loop ve Softbuffer fallback renderer ile native pencereyi yonetir.
10. Tauri WebView VM framebuffer render etmez.
11. RFB endpoint v0.5.0'dan beri 127.0.0.1 disina acilmaz.
12. Gamepad capability v0.9.0 ile TurkuazDisplay icindeki GilRs adapterinden gercek local observation alir.

Gaming renderer gecisinde RFB/Softbuffer adapterleri degisebilir; Domain, Engine API ve Desktop use-case siniri korunur.

## QMP session state

AwaitingGreeting -> CapabilitiesNegotiated -> Ready

## VM state

Created -> Stopped -> Starting -> Running -> Stopping -> Stopped

Pause/resume state gecisleri domain seviyesinde tanimlidir ancak use-case olarak sonraki surumlere birakilmistir.

## Config kurali

Deployment degerleri config/turkuazvm.yml dosyasinda tanimlanir ve settings modellerine disaridan verilir. Adapter icinde deployment config hard-code edilmez.

## Persistence kurali

Core domain serde annotation tasimaz. YAML persistence repository adapterinin sorumlulugudur. Runtime disk path'leri manifestte relative tutulur.

## Android Runtime vertical slice

1. `GuestProfile::Android` VM aggregate icinde yalniz guest siniflandirmasini tasir; Core ADB bilmez.
2. AndroidApplicationService VM/Network bounded context ile Android bounded context'i Engine seviyesinde orkestre eder.
3. Android configure yalniz Stopped VM icin calisir ve dedicated `android-nat` binding'i provision eder.
4. ADB host endpoint yalniz loopback araligindan otomatik ayrilir.
5. AndroidRuntimeService profile/readiness/display use-case'lerini yonetir.
6. AndroidPackageService relative APK path ve canonical package-root containment kuralini uygular.
7. AndroidInputService typed tap/swipe/key/text actionlarini AndroidInputPort'a aktarir.
8. AdbRuntimeTool gercek `adb` process adapteridir ve `guest` Tool katmaninda kalir.
9. YamlAndroidProfileRepository Android profilini `android.yml` schema version 1 ile `machine.yml` disinda saklar.
10. Engine API v5 Android profile/status/package/input DTO'larini Desktop'a acar; bu contract v0.7.0 Engine API v7 icinde korunur.
11. Desktop Android modal yalniz Engine API use-case'lerini kullanir; ADB veya QEMU cagiramaz.

## Gaming GPU vertical slice

1. GamingGpuService host ve QEMU GPU capability probe sonucunu tek raporda birlestirir.
2. NativeGpuProbeTool host Vulkan/OpenGL runtime varligini adapter katmaninda kontrol eder.
3. QemuGpuProbeTool kurulu QEMU binary icinde virtio-gpu, VirGL/Venus ve rutabaga/GfxStream capability'lerini introspection ile dogrular.
4. GpuBackendPreference::Auto deneysel GfxStream yolunu explicit opt-in olmadan secmez.
5. Stock QEMU accelerated VirGL/Venus ve rutabaga/GfxStream yolu v0.7.0'da Linux host ile sinirli tutulur; Windows guvenli virtio 2D/software fallback kullanir.
6. QemuCommandBuilder secilen typed GPU backend'i QEMU device/display launch spec'ine cevirir.
7. Engine API v7 icinde korunan `GetGpuCapabilities` ile istenen ve gercek secilen backend bilgisini Desktop'a acar.
8. Desktop GPU paneli Vulkan loader, QEMU device/property capability'leri ve experimental durumunu ayri ayri gosterir.
9. TurkuazDisplay FPS yaninda ortalama frame-time telemetry de raporlar.
10. GPU bounded context QEMU, Tauri, serde veya process implementasyonunu import etmez.

## v0.8.0 Gaming Input bounded context

1. `turkuazvm-gaming-input` keymapping domain, capability ve stateful translator katmanini tasir.
2. `YamlGameInputProfileRepository` persistence adapteridir; domain serde/YAML bilmez.
3. Editor path: Desktop View -> Tauri Controller -> DesktopService -> Engine API -> GamingInputApplicationService.
4. Runtime path: TurkuazDisplay -> bounded GamingInputBridgeTool -> local Engine API -> translator.
5. ADB yalniz tekil Tap/AndroidKey fallback sink'idir; persistent multi-touch Guest Agent sink'ine ayrilir.
6. Host gamepad observation port ile soyutlanir; concrete adapter v0.9.0 ile TurkuazDisplay tarafinda eklenir.

## v0.9.0 Game Profiles & Compatibility bounded context

1. `turkuazvm-game-catalog` oyun kimligi, package eslestirmesi, runtime gereksinimi ve onerilen input profilini typed domain olarak tasir.
2. `YamlGameCatalogRepository` YAML/serde detaylarini adapter sinirinda tutar; Service magic string parse etmez.
3. Engine API v8 katalog listesi, kurulu oyun tespiti, compatibility raporu ve profile apply use-case'lerini acar.
4. `turkuazvm-guest-agent-protocol` persistent multi-touch icin versioned protocol contractini tasir.
5. `AndroidGuestAgentTool` ADB forward ile ayri loopback input-agent endpoint'i acar; capability yalniz gercek handshake ile READY olur.
6. AOSP referans `TurkuazInputAgent` AccessibilityService, devam eden stroke state'ini koruyarak full touch frame'lerini Android gesture katmanina uygular.
7. TurkuazDisplay concrete GilRs adapteri ile Windows/Linux gamepad eventlerini normalized button/axis eventlerine cevirir.
8. Game Catalog profili compatibility ve input/runtime ayari tasir; emulator identity veya anti-cheat bypass davranisi tasimaz.


## v0.10.0 Android Image bounded context

1. `turkuazvm-android-image` AOSP source/build tool detaylarindan bagimsiz image profile, artifact, capability, state ve assignment domain modellerini tasir.
2. `AndroidImageRepositoryPort` registry/assignment persistence siniridir; YAML Serde yalniz repository adapterindedir.
3. `AndroidImageBuilderPort` build plan ve artifact registration siniridir; shell/process mantigi Domain'e girmez.
4. Engine AOSP build'i request thread'i icinde calistirmaz; yalniz build planini uretir ve disarida uretilen bundle'i register eder.
5. `AospAndroidImageTool` Linux host/source/script capability'sini raporlar ve bundle artifactlerini SHA-256 ile fingerprint eder.
6. `prepare_aosp_source.sh` android-latest-release Repo checkout'unu hazirlar; build script custom Cuttlefish product ve TurkuazInputAgent'i image'a dahil eder.
7. READY state icin boot, super, userdata, bootloader ve composite disk artifactleri zorunludur.
8. Image assignment yalniz Stopped Android VM icin yapilir ve provisioning state `PendingFirstBoot` ile baslar.
9. Engine API v9 image list/define/build-plan/register/assign/get-assignment contractlarini Desktop'a acar.
10. Desktop AOSP veya QEMU'yu dogrudan cagirmadan image manager ve VM assignment akisini Engine API uzerinden yonetir.
11. Symlink artifactler register edilmez ve ayni image ID ikinci Define ile mevcut registry kaydini ezemez.
12. v0.11.8 Android Boot Integration assignment/provisioning state uzerinden tamamlanir.

## v0.11.8 Android Boot Integration

1. Core `VmRuntimeMediaPlan` hypervisor adapterine guest-specific boot medyasini typed contract ile tasir.
2. Android registered `composite.img` image template olarak checksum/size preflight'ten gecer.
3. `AndroidRuntimeMediaTool` her VM icin ayri QCOW2 overlay ve pflash state hazirlar.
4. `QemuCommandBuilder` Android bootloader/pflash ve composite overlay'i generic VM disklerinden ayri launch spec'e cevirir.
5. Ilk Start ADB readiness, display provisioning ve Guest Agent readiness zincirini Engine Service seviyesinde orkestre eder.
6. Guest Agent provisioning mevcut AccessibilityService listesini koruyarak Turkuaz service componentini etkinlestirir.
7. Assignment schema 2 boot attempt sayisini ve first-boot hata durumunu saklar; schema 1 kayitlari okunabilir kalir.
8. First-boot hatasinda assignment Failed olur ve VM kontrollu stop edilir.
9. Ready assignment sonraki startlarda tekrar first-boot provisioning yapmaz.
10. Engine API v10 assignment boot attempt bilgisini Desktop'a acar.

## v0.12.0 Android Auto Provisioning

1. `AndroidImageDistributionPort` hazir resmi dagitim kaynagini Android Image Service'ten ayirir.
2. `AndroidCiDistributionTool` latest Cuttlefish x86_64 buildini kesfeder ve ayni build device/host paketlerini staging alanina indirir.
3. Uzun indirme Engine API request thread'inde bekletilmez; application service background worker baslatir.
4. Image lifecycle `Defined -> Installing -> Ready/Failed` olarak versionlanir.
5. YAML registry schema 2 Installing state'ini tasir; schema 1 backward read korunur.
6. Repository clone'lari ortak mutex ile list/save yarismasini engeller; manifest temp/backup/rename ile commit edilir.
7. Rust composite tool Android sparse partitionlari acar ve GPT/U-Boot composite diski binary yardimci gerektirmeden uretir.
8. Stok Cuttlefish image Guest Agent capability'sini false tasir; first boot Guest Agent provisioning'i capability bazli kosulludur.
9. Engine API v13 install distribution actionini Desktop'a acar; Desktop INSTALLING state'ini periyodik izler.
10. Engine restartinda yarida kalan install kaydi Failed durumuna toparlanir ve yeniden denenebilir.

## Sonraki adim

v0.13.0 Android Gaming Renderer ile accelerated Android display yolunu harden etmek.
