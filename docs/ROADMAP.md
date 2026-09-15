# 📄 Dosya Yolu: /turkuazvm/docs/ROADMAP.md
# 📌 Amac: TurkuazVM surum kilometre taslarini ve mimari ilerleme sirasini kilitler
# 📌 Modul - Markdown
# Version: 0.41.2
# Aciklama: Tarihsel temel kilometre taslarini ve aktif v0.27-v0.41.2 release hattini tek roadmap altinda tutar
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

# TurkuazVM Roadmap

## Aktif Surumleme Notu

Bu dosyadaki v0.1-v0.12 bolumleri tarihsel foundation planini korur. v0.27 ve sonrasi icin ayrintili gercek degisiklik kaynagi `CHANGELOG.md` dosyasidir. Eski `Planlandi` etiketleri aktif release durumunu temsil etmez.

## Aktif Release Hatti

- v0.27.x - Managed Network ve Connection UX: Tamamlandi.
- v0.28-v0.30 - Installer download control ve Create Wizard UX: Tamamlandi.
- v0.31.x - Step-by-step Create Wizard ve Windows network hardening: Tamamlandi.
- v0.32.x - Portable QEMU Turkuaz NAT, persistent disk boot ve Connection Center: Tamamlandi.
- v0.33.x - Desktop Workspace redesign ve launcher workspace gate: Tamamlandi.
- v0.34.0 - Android Surum Merkezi ve Gorev Cubugu: Tamamlandi.
- v0.35.0 - Android Catalog ve Standard Mode UX: Tamamlandi.
- v0.36.x - Goruntu Merkezi, download source tek-otoritesi ve compile/launcher hotfixleri: Tamamlandi.
- v0.37.0 - Workspace UX ve Release Hardening: Tamamlandi.
- v0.37.2 - Windows Header Path Validation Hotfix: Tamamlandi.
- v0.37.3 - Windows Cargo Process Tree Launcher Hotfix: Tamamlandi.
- v0.37.6 - Android 11 BUILD_INFO Compatibility: Tamamlandi.
- v0.37.7 - Android 11 Rust Dead-Code Compile Hotfix: Tamamlandi.
- v0.37.8 - Android Download UX Telemetry: Tamamlandi.
- v0.37.9 - Android 10 Legacy CI Discovery: Tamamlandi.
- v0.38.0 - Unified Download UX: Tamamlandi.
- v0.39.0 - Linux Edition Media + Android Provider Router.
- v0.39.3 - Android Legacy CI Artifact Probe Hotfix: Tamamlandi.
- v0.39.5 - Engine API Timeout Safety + Linux Installer Media Policy: Tamamlandi.
- v0.39.6 - Launcher Config Drift Hotfix: Tamamlandi.
- v0.39.7 - Workspace Layout Hotfix: Tamamlandi.
- v0.40.0 - Official Source Resolver Architecture: Tamamlandi.
- v0.40.1 - Managed Installer Download Hotfix: Tamamlandi.
- v0.40.2 - Native Display Telemetry + CPU Fallback Render Patch: Tamamlandi.
- v0.40.3 - One-Shot Installer ISO Session Lifecycle: Tamamlandi.
- v0.40.4 - Launcher Softbuffer Contract Hotfix: Tamamlandi.
- v0.40.10 - Engine StartVm Crash Diagnostics Hotfix: Tamamlandi.
- v0.40.14 - Android Emulator Port Config Wiring Hotfix: Runtime hotfix temeli.
- v0.41.2 - Baglanti Merkezi Calisan-VM Erisim Yamasi: Aktif release; running Turkuaz NAT VM icin SSH/RDP host yayinini onayli stop/publish/restart ile hazirlar.
- v0.41.1 - Workspace Kullanilabilirlik Yamasi: Korunan onceki release; 1920x1080 olcekleme, tek bos-filo onboarding ve genis Goruntu Merkezi.
- v0.41.0 - VM Kontrol Merkezi: Korunan onceki release; VM yonetim ekranini kontrol-oncelikli workspace modeline tasir.
- v0.40.13 - Windows Android SDK System Image Provider: Korunan onceki release.
- v0.40.12 - Windows PowerShell 5.1 Process Collection Hotfix: Korunan onceki release.
- v0.40.11 - Windows Runtime Binary Lock Hotfix: Korunan onceki release.
- v0.40.9 - Android CI Numeric Build-ID Compile Hotfix: Korunan onceki release.
- v0.40.8 - Android CI Launcher Verifier Contract Hotfix: Korunan onceki release.
- v0.40.7 - Android CI Dead-Code Compile Hotfix: Korunan onceki release.
- v0.40.6 - Release Metadata Contract Hotfix: Korunan onceki release.
- v0.40.5 - Android Historical CI Target Discovery Hotfix: Korunan onceki release.
- v0.37.1 - Windows PowerShell 5.1 Native Process Hotfix: Tamamlandi.

### v0.37.1 Kilitli Kapsam

- v0.37.0 Home ve inline VM Workspace korunur.
- Native process stderr/stdout PowerShell hata semantiginden ayrilir.
- rustup/rustc/cargo/QEMU/WinGet exit-code tabanli Tool kullanir.
- Exact Rust toolchain policy `rust-toolchain.toml` kaynagindan okunur.

Durum: Statik validation tamamlandi; Windows Cargo/WHPX host retest bekliyor.

### v0.37.0 Kilitli Kapsam

- Home dashboard.
- Inline VM Disk/Ag/Erisim/Snapshot/Gunluk workspace.
- View ve workspace CSS modulerlesmesi.
- Standart/Uzman progressive disclosure.
- Global launcher Administrator elevation temizligi.
- Rust 1.98.0 toolchain pin.
- Runtime-generated version header tek-otoritesi.
- Artifact Cache DNS-rebind ve IPv4-mapped IPv6 SSRF hardening.
- Regression testlerinin eski implementasyon tokenlari yerine davranis kontratina migrate edilmesi.

Durum: Tamamlandi; v0.37.1 native-process hotfix ile supersede edildi.

## v0.1.0 - Core Bootstrap

- Cargo workspace
- Core/QEMU/platform/engine sinirlari
- Host capability detection
- QEMU discovery

Durum: Tamamlandi.

## v0.1.1 - QMP Core

- QMP greeting
- qmp_capabilities
- query-version
- query-commands
- Request id correlation
- Async event ayiklama
- TCP transport

Durum: Tamamlandi.

## v0.1.2 - VM Lifecycle

- VirtualMachine aggregate
- VmState state machine
- Create/Start/Stop commandlari
- VmLifecycleService
- VmRepositoryPort
- HypervisorRuntimePort
- QEMU process registry
- QMP process termination
- Failure state ve compensation

Durum: Tamamlandi.

## v0.1.3 - Storage Foundation

- Disk domain modelleri
- StoragePort
- qemu-img adapteri
- QCOW2/RAW create
- JSON disk info
- Grow-only disk resize
- Disk attach modeli
- YamlVmRepository
- machine.yml manifest persistence
- QEMU launch disk attachment

Durum: Tamamlandi.

## v0.1.4 - Network Foundation

- NAT network profile
- Port forwarding domain modeli
- NetworkPort
- Windows/Linux bridge adapter sinirlari
- QEMU user-mode network builder

Durum: Tamamlandi.

## v0.1.5 - Guest Boot

- ISO attach
- Firmware secimi
- Boot order
- UEFI/BIOS profile
- Visible/default QEMU display
- Ilk boot edilebilir Linux/Windows guest launch spec

Durum: Tamamlandi.

## v0.1.6 - Platform Networking

- Windows existing TAP adapter preflight
- Linux existing TAP / bridge preflight
- Linux bridge helper strategy
- Linux managed TAP fallback
- Network runtime lease ve PID binding
- Adapter cleanup ve crash recovery

Durum: Tamamlandi.

## v0.2.0 - Desktop

- Tauri manager UI
- Versioned local Engine API
- Engine auto-start
- VM listesi
- Basic create wizard
- Start/Stop controls
- Runtime event updates

Durum: Tamamlandi.

## v0.3.0 - Snapshot ve Clone

- Offline QCOW2 snapshot service
- Full clone service
- Linked clone backing image strategy
- Snapshot/clone Desktop UI
- machine.yml schema version 5

Durum: Tamamlandi.

## v0.4.0 - Remote Host

- Headless Engine
- Authenticated Engine API v3
- TLS remote transport
- Local/remote host profiles
- Windows Desktop -> Linux Engine
- systemd deployment foundation

Durum: Tamamlandi.

## v0.5.0 - Native Display ve Input

- Native render window
- Loopback RFB framebuffer fallback
- Raw mouse input
- Keyboard capture
- Fullscreen
- Frame pacing temel altyapisi
- Engine API v4 display session
- Tauri disinda ayri TurkuazDisplay process

Durum: Tamamlandi.

## v0.6.0 - Android Runtime Foundation

- Ayrilmis Android bounded context
- Android guest profile
- ADB discovery/capability adapteri
- Otomatik loopback ADB NAT provisioningi
- android.yml schema version 1
- Device readiness/status
- Display width/height/density profile
- APK install/uninstall
- Package list/launch/stop
- Input injection port foundation
- Desktop Android Runtime paneli

Durum: Tamamlandi.

## v0.7.0 - Gaming GPU

- Ayrilmis Gaming GPU bounded context
- Host Vulkan/OpenGL capability probe
- QEMU virtio-gpu / VirGL / Venus / rutabaga / GfxStream probe
- Safe auto backend policy
- Experimental Android GfxStream opt-in guard
- QEMU GPU launch strategy
- Engine API v6 GPU capability contracti
- Desktop GPU capability paneli
- FPS ve frametime telemetry

Durum: Tamamlandi.

## v0.8.0 - Gaming Input

- Ayrilmis Gaming Input bounded context
- Normalized keymapping profile
- WASD virtual joystick translator
- Relative raw mouse look translator
- Desktop keymapping editor
- TurkuazDisplay -> local Engine raw input bridge
- ADB tap/key fallback
- Persistent multi-touch guest-agent capability siniri
- HostGamepadPort foundation

Durum: Tamamlandi.

## v0.9.0 - Game Profiles & Compatibility

- Oyun profil katalogu
- Package/profile eslestirme
- PUBG Mobile deneysel compatibility profile
- Compatibility database foundation
- Android Guest Agent persistent multi-touch transport
- AOSP referans AccessibilityService agent
- Concrete GilRs Windows/Linux host gamepad adapteri
- Desktop Game Catalog ve compatibility UI

Durum: Tamamlandi.


## v0.10.0 - Turkuaz Android Image Foundation

- Ayrilmis Android Image bounded context
- AOSP android-latest-release source preparation scripti
- Linux build host ve 400 GiB disk preflight
- Cuttlefish x86_64 custom Turkuaz product
- TurkuazInputAgent product image entegrasyonu
- External AOSP build plan
- SHA-256 artifact registry
- Android image manifest schema 1
- READY image assignment
- Engine API v9
- Desktop Android Images manager

Durum: Tamamlandi.

## v0.11.0 - Android Boot Integration

- Assigned image artifact -> typed QEMU runtime media plan
- Cuttlefish GPT composite disk ve U-Boot/pflash boot zinciri
- boot/super/userdata/bootloader/composite artifact preflight
- VM ve image-ozel QCOW2 overlay ile immutable image template
- First-boot provisioning ve boot attempt state
- ADB display readiness
- AccessibilityService provisioning + Guest Agent Ping/Pong readiness
- Assignment schema 2 ve schema 1 backward read
- Engine API v10 ve Desktop boot attempt gorunumu
- First-boot failure recovery ve kontrollu VM stop

Durum: Tamamlandi.

## v0.11.1 - Windows Test Launcher

- Tek tik `TurkuazVM-Start.cmd` girisi
- Rust/Cargo, Visual Studio, QEMU ve WHPX preflight
- Launcher transcript loglari
- Desktop test build/start zinciri

Durum: Tamamlandi.

## v0.11.2 - Windows Dependency Bootstrap

- Eksik QEMU, ADB ve WebView2 icin kullanici onayli WinGet bootstrap
- QEMU ve WinGet portable PATH yeniden discovery
- WebView2 resmi registry detection
- Windows Hypervisor Platform feature state ve enable akisi
- Firmware virtualization preflight

Durum: Tamamlandi.

## v0.11.3 - WHPX Runtime Probe Patch

- Firmware virtualization WMI sonucu tanisal bilgi olarak kalir.
- QEMU `-accel whpx` runtime probe gercek acceleration availability kararini verir.
- HypervisorPresent ayri preflight bilgisi olarak gosterilir.

Durum: Tamamlandi.

## v0.11.4 - Windows Header Path Verifier Patch

- Windows path separator normalization duzeltildi.
- `.gitignore` regression probe eklendi.

Durum: Tamamlandi.

## v0.11.5 - Desktop Game Profile Verifier Patch

- Statik Desktop DOM contractlari `index.html` uzerinden dogrulanir.
- Dinamik `data-game-action` contractlari `app.js` uzerinden dogrulanir.

Durum: Tamamlandi.

## v0.11.6 - Softbuffer Display Compile Patch

- TurkuazDisplay `softbuffer 0.4.8` API contractina guncellendi.
- `Surface::buffer_mut()` ve row-major `u32` framebuffer yazimi kullanilir.
- Eski `Pixel`, `next_buffer`, `pixels_iter` API contractlari verifier ile yasaklanir.

Durum: Tamamlandi.

## v0.11.9 - Build Warning Cleanup Patch

Durum: Tamamlandi

- Gercek Windows build warningleri temizlendi.
- Obsolete Engine controller scaffoldlari kaldirildi.
- Error Display contractlari ve dead-code deny kapisi eklendi.
- Runtime davranisi degistirilmeden Android first-boot testine hazirlik tamamlandi.

## v0.11.8 - Tauri Windows Icon Build Patch

- Tauri Windows resource build icin gecerli `icons/icon.ico` asseti
- Tauri bundle config icinde acik icon referansi
- Build oncesi ICO varlik ve temel header verifier kapisi

Durum: Tamamlandi.

## v0.12.0 - Android Auto Provisioning

- Resmi Android CI latest Cuttlefish x86_64 build kesfi
- Ayni build device image + host package otomatik indirme
- Background INSTALLING lifecycle ve Desktop polling
- SHA-256 provenance, safe archive preflight ve staging rollback
- Rust sparse image + GPT composite assembler
- Stok Cuttlefish capability ayrimi ve Guest Agent olmayan first-boot yolu
- Image registry schema 2 ve config schema 10
- Engine API v11 install distribution action

Durum: Tamamlandi.

## v0.12.1 - Android Auto Provisioning Hardening

- Install progress + log
- Retry/resume download
- Minimum free disk preflight
- Engine API v12
- Config schema 11

Durum: Tamamlandi.

## v0.12.2 - Android Provisioning Operations

- Kurulum iptal / retry / staging cleanup
- Download hiz ve ETA telemetry
- Local install.log goruntuleme
- READY image -> durmus Android VM tek tik assignment
- Engine API v13

Durum: Tamamlandi.

## v0.12.3 - Android First-Boot Request Timeout Patch

- Android first-boot icin request bazli long timeout
- ADB readiness + Guest Agent provisioning sirasinda Desktop timeout korumasi
- Uzun storage/runtime operasyonlari icin ortak timeout sinifi
- Config schema 12
- Engine API v13 korunur

Durum: Tamamlandi.


## v0.12.4 - Storage + Network Foundation

- Cross-platform host storage capacity probe
- Desktop Storage management foundation
- Desktop Network capability/User NAT foundation
- Engine API v14 storage/network actions
- Config schema 13 storage/network defaults
- `.tvmimg` ZIP64 portable image format specification
- Private-copy runtime disk policy
- Android shared backing overlay -> backing-free private QCOW2 migration

Durum: Tamamlandi.


## v0.12.5 - Portable Image Interop

- `.tvmimg` pack/export service ve tool adapteri
- `.tvmimg` import + manifest/checksum validation
- Standart QCOW2/RAW import/export
- VMDK/VDI import icin qemu-img convert adapteri
- Varsayilan private-copy install; shared backing yalniz explicit clone modunda

Durum: Tarihsel plan; aktif surumleme tarafindan superseded. Guncel kapsam icin CHANGELOG.md esas alinir.

## v0.13.0 - Android Gaming Renderer

- Android accelerated renderer capability matrix
- Linux accelerated path hardening
- Windows renderer stratejisi
- Frame latency ve present telemetry
- Safe software/2D fallback korunumu

Durum: Tarihsel plan; aktif surumleme tarafindan superseded. v1.0 hedefi once aktif v0.x release hattinin native host gate ile kilitlenmesini bekler.

## v1.0.0 - TurkuazVM

- Genel VM yonetimi
- Windows/Linux host
- Android Gaming Runtime
- Turkuaz Android/AOSP image build ve provisioning
- Registered Android image boot integration
- ARM/ARM64 ABI compatibility/translation karari
- Windows gaming renderer acceleration yolu
- Oyun compatibility test matrisi
- Stable migration ve backup akislari

