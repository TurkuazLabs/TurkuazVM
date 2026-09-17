# 📄 Dosya Yolu: /turkuazvm/RELEASE_NOTES_v0.41.6.md
# 📌 Amac: TurkuazVM v0.41.6 GitHub release notlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.41.6
# Aciklama: Windows kurulum dosyalarini TurkuazLabs/TurkuazVM marka hiyerarsisine tasiyan installer patchini ozetler
# Bagimli Oldugu Katman: Tool | View | CI/CD

# TurkuazVM v0.41.6

v0.41.6, Windows installer icin marka klasor hiyerarsisini duzeltir. Kurulu uygulama dosyalari artik dogrudan `TurkuazVM` klasoru yerine `TurkuazLabs/TurkuazVM` altinda tutulur.

## Windows Install Root

- System-wide kurulum varsayilani: `C:\Program Files\TurkuazLabs\TurkuazVM`.
- Current-user kurulum varsayilani: `%LOCALAPPDATA%\TurkuazLabs\TurkuazVM`.
- Tek Setup EXE icindeki `/CurrentUser` ve `/AllUsers` modlari korunur.
- Registry scope ve uninstall davranisi degismez.

## Runtime Data

- Writable runtime/config kokunun mevcut davranisi korunur: `%LOCALAPPDATA%\TurkuazVM`.
- Bu patch kullanici verisini tasimaz veya yeniden adlandirmaz; yalniz kurulu uygulama binary/resource kokunu markali klasor hiyerarsisine alir.
- Portable paket klasor-ici davranisini korur.

## Installer Build Guvenilirligi

- Windows dagitim workflow'u Tauri CLI `2.11.4` surumune pinlendi.
- Packaging scripti ayni surumun resmi Tauri NSIS template'ini indirir.
- Template Git blob SHA-1 degeri ile fail-closed dogrulanir.
- Yalniz install-root ile ilgili NSIS satirlari patch edilir; beklenen satir sayisi degisirse build durur.
- Gecici patchlenmis NSIS template build sonunda silinir ve repository'ye girmez.

## CI / Dogrulama

- Statik v0.41.6 install-root kontrati eklendi.
- Gercek Windows smoke testi current-user kurulumunun tam olarak `%LOCALAPPDATA%\TurkuazLabs\TurkuazVM` altina kuruldugunu kontrol eder.
- Gercek Windows smoke testi all-users kurulumunun tam olarak Program Files altindaki `TurkuazLabs\TurkuazVM` klasorune kuruldugunu kontrol eder.
- AppData runtime materialization ve uninstall sonrasi kullanici verisinin korunmasi tekrar test edilir.

## Uyumluluk

- Breaking change yoktur.
- Config schema 22 korunur.
- Download sources schema 5 korunur.
- Engine API version 24 korunur.
- VM manifest schema 6 korunur.
- Guest catalog schema 4 korunur.
- Android image manifest schema 4 korunur.
- Guest agent protocol version 2 korunur.
- QEMU installer icine gomulu degildir; mevcut dependency/runtime katmani tarafindan erisilebilir olmalidir.
