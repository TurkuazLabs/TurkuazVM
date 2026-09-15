# 📄 Dosya Yolu: /turkuazvm/RELEASE_NOTES_v0.41.3.md
# 📌 Amac: TurkuazVM v0.41.3 GitHub release notlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.41.3
# Aciklama: CI host bagimliliklari, Windows ortam degiskeni cozumlemesi, Tauri asset ve GitHub governance duzeltmelerini ozetler
# Bagimli Oldugu Katman: Tool | View | Config

# TurkuazVM v0.41.3

v0.41.3 bir patch release adayidir. v0.41.2 urun davranisini korurken gercek Windows/Linux CI dogrulamasinda ortaya cikan host ve paketleme sorunlarini kapatir ve GitHub gelistirme akislarini standartlastirir.

## Duzeltmeler

- Windows CI host dependency Tool katmani ortam degiskenlerini case-insensitive cozer; `%ProgramFiles%` ile `PROGRAMFILES` farki QEMU PATH cozumunu bozmaz.
- Linux compiler profiline Tauri GTK/WebKit bagimliliklari ve `libudev-dev` eklenmistir.
- Tauri buildinin bekledigi `apps/desktop/src-tauri/icons/icon.png`, mevcut uygulama ikon kaynagindan geri getirilmistir.
- Rust workspace Windows ve Linux GitHub runnerlarinda gercek Cargo derlemesiyle dogrulanir.
- Runtime validation Windows ve Linux tarafinda gercek QEMU smoke akisini calistirir.

## GitHub Gelistirme Akisi

- CODEOWNERS eklendi.
- Pull Request kalite ve mimari kontrol sablonu eklendi.
- CONTRIBUTING ve SECURITY politikalari eklendi.
- Bug ve feature issue formlari standartlastirildi.
- Bos issue acma kapatildi; tanimli formlar kullanilir.

## Uyumluluk

- Breaking change yoktur.
- Config schema 22 korunur.
- Engine API version 24 korunur.
- VM manifest schema 6 korunur.
- Guest agent protocol version 2 korunur.

## Release Gate

v0.41.3 tag ve GitHub Release olusturulmadan once release PR'indeki tum CI kontrolleri ve merge sonrasi `main` CI turu PASS olmalidir.
