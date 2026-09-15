# 📄 Dosya Yolu: /turkuazvm/docs/recovery/A_G_RECOVERY_MATRIX.md
# 📌 Amac: Kayip v0.13.1 A-G kaynak arsivi yerine eldeki kanitlardan kurtarilabilen ve kurtarilamayan degisiklikleri kaydeder
# 📌 Modul - Markdown
# Version: 0.27.1-recovered.1
# Aciklama: Calisan v0.12.4 tabani ile A-G audit kanitlari arasindaki farki fail-closed olarak belgeler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View | Language

# A-G Recovery Matrix

## Temel karar

Bu FULL paket calisan `v0.12.4` kaynak agacini canonical runtime tabani olarak korur.

Kayip `v0.13.1-G` ZIP'i mevcut olmadigi icin A-G kaynak dosyalari tahmin edilerek yeniden uretilmemistir. Yalniz elde fiziksel olarak bulunan H-I-J-K-L araclari ve evidence dosyalari ana workspace'e eklenmistir.

Bu karar calisan runtime'i, elde olmayan bir branch'in tahmini source koduyla bozmayi engeller.

## Eldeki A-G kanitlari

Eski validation kayitlari v0.13.1-G icin su source shape'i bildiriyordu:

- 378 toplam dosya
- 242 Rust dosyasi
- 14 Cargo manifest
- 13 workspace member
- 23 Rust test declaration
- Static gate PASS
- Actual Cargo gate UNVERIFIED/BLOCKED

## A+B kanitlanmis hedefler

Eski audit validation kayitlarina gore A+B hattinda asagidaki davranislar hedeflenmis ve static olarak PASS raporlanmisti:

- TVGB v2 directional HMAC-SHA256
- H2G/G2H ayri session key
- Full authenticated transcript
- Per-frame auth tag
- Replay window
- Header/payload tamper reject
- Persistent runtime controller wiring
- Continuous QEMU runtime monitor
- QMP ve guest heartbeat issue sink
- Corrupt runtime record isolation
- Diagnostic redaction
- Secret Debug redaction ve memory cleanup
- HostToGuest transfer direction/size validation

Bu source dosyalari mevcut recovery girdilerinde bulunmadigi icin bu pakette A+B runtime feature parity iddia edilmez.

## C kanitlanmis hedefler

- QMP next_sequence cursor
- Journal append outcome + repair
- Rotation/retention policy
- Critical-event non-drop/resync
- VmStopIntent coordination
- Durable recovery trigger consumer

Kayip source arsivi olmadan bu kodlar active v0.12.4 runtime'a enjekte edilmemistir.

## D kanitlanmis hedefler

- Signed Guest Agent package trust
- Concrete offline injection
- Windows service wrapper
- Linux service race hardening
- Android persistent service/SELinux hardening
- Clipboard broker IPC
- Online update rollback

Kayip source arsivi olmadan bu kodlar active v0.12.4 runtime'a enjekte edilmemistir.

## E-F-G kanitlanmis gate hedefleri

- Windows/Linux Cargo check/test
- Rust toolchain pinning
- Compile API proxy
- Win32 CreateFileW HANDLE `null_mut()` duzeltmesi

Bu paket v0.12.4 workspace'inin kendi dependency/API setini korur. v0.13.1-G'ye ozgu Win32 dosyalari bu tabanda mevcut olmadigi icin o patch burada uygulanamaz.

## H-L durumu

H-I-J-K-L dosyalari elde fiziksel olarak bulundugu icin FULL source tree'ye eklenmistir. Bunlar Rust v0.12.4 runtime'inin yerine gecmez; compiler evidence, host resilience policy/runtime tooling ve runtime validation harness saglar.

## Release iddiasi

- FULL source tree: VAR
- Calisan v0.12.4 runtime source: KORUNDU
- H-L tooling: ENTEGRE
- Kayip A-G exact source parity: YOK
- Static recovery gate: calistirilabilir
- Gercek Cargo check/test: toolchain ve dependency erisimi olan hostta zorunlu
