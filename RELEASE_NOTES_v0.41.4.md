# 📄 Dosya Yolu: /turkuazvm/RELEASE_NOTES_v0.41.4.md
# 📌 Amac: TurkuazVM v0.41.4 GitHub release notlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.41.4
# Aciklama: Ilk gercek Windows NSIS/portable dagitim zincirini, packaged runtime kok sabitlemesini ve compiler-stage ayrimini ozetler
# Bagimli Oldugu Katman: Tool | View | Config | CI/CD

# TurkuazVM v0.41.4

v0.41.4, TurkuazVM icin ilk dogrudan kullanilabilir Windows dagitim pipeline'ini ekler. v0.41.3 CI/release hardening ve v0.41.2 Baglanti Merkezi davranisi korunur.

## Windows Dagitimi

- GitHub Actions Windows runner uzerinde gercek Tauri release build ve NSIS paketleme yapilir.
- `TurkuazVM-0.41.4-x64-Setup.exe` current-user kurulum modeliyle uretilir.
- `TurkuazVM-0.41.4-x64-Portable.zip` kurulum gerektirmeyen alternatif paket olarak uretilir.
- `TurkuazVM-0.41.4-SHA256SUMS.txt` Setup ve Portable paketlerinin SHA-256 degerlerini tasir.
- Installer Turkce ve Ingilizce dil secimini destekler.
- Desktop ile birlikte release `turkuazvm-engine.exe` ve `turkuazvm-display.exe` runtime stage'e dahil edilir.
- Dagitim configindeki debug binary yollari `bin/turkuazvm-engine.exe` ve `bin/turkuazvm-display.exe` olarak yeniden yazilir.
- `v*` tag kosularinda Windows artefactlari GitHub Release Assets'e otomatik yuklenir.

## Paketli Runtime Duzeltmesi

- Paketli TurkuazVM, EXE'nin yaninda `config/turkuazvm.yml` bulunduğunda runtime calisma kokunu EXE dizinine sabitler.
- Bu davranis Start Menu veya farkli bir working-directory uzerinden acilista config, `bin`, `data` ve relative runtime yollarinin yanlis kokten cozulmesini engeller.
- `TURKUAZVM_CONFIG` acikca verilmis ise mevcut override davranisi korunur.

## CI / Compiler Gate Duzeltmesi

- Windows'a ozel Tauri config artik gecici `distribution/windows/stage` klasorunu normal Cargo build kontratina dahil etmez.
- Packaging Tool, resource mapping'i sadece paketleme aninda gecici Tauri config ile ekler ve build sonunda temizler.
- Boylece `cargo check --workspace --all-targets` paketleme stage'i olusturulmadan da Windows ve Linux runnerlarda PASS olabilir.

## Uyumluluk

- Breaking change yoktur.
- Config schema 22 korunur.
- Engine API version 24 korunur.
- VM manifest schema 6 korunur.
- Guest agent protocol version 2 korunur.
- Installer simdilik `currentUser` modundadir; Program Files/system-wide kurulum AppData tabanli runtime data root ayrimi gelmeden acilmamistir.
- QEMU bu release ile installer icine gomulmez; mevcut TurkuazVM dependency/runtime katmaninin QEMU'yu sistemde erisebilir hale getirmesi gerekir.

## Dogrulama

Release PR uzerinde Windows Distribution, Compiler Log Gate, Runtime Validation, Host Resilience Policy/Runtime ve artifact-cache agent gate PASS olmalidir. Tag sonrasinda Setup, Portable ve SHA-256 dosyalarinin GitHub Release Assets'e yuklendigi ayrica dogrulanir.
