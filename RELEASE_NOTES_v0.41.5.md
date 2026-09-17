# 📄 Dosya Yolu: /turkuazvm/RELEASE_NOTES_v0.41.5.md
# 📌 Amac: TurkuazVM v0.41.5 GitHub release notlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.41.5
# Aciklama: AppData runtime-root, dual NSIS install-mode, gercek installer smoke ve Turkce MultiUser localization degisikliklerini ozetler
# Bagimli Oldugu Katman: Tool | View | Config | CI/CD

# TurkuazVM v0.41.5

v0.41.5, Windows kurulu paketinin writable runtime/config verisini kurulum dizininden ayirir ve tek NSIS installer icinde current-user ile system-wide kurulum modlarini birlikte destekler.

## Windows Runtime Data Root

- Kurulu Windows Desktop ilk acilista kullaniciya ait runtime/config kokunu `%LOCALAPPDATA%\TurkuazVM` altinda materialize eder.
- `data/`, `packages/`, loglar ve kullanici tarafindan degistirilebilir download ayarlari writable kullanici alaninda kalir.
- Engine/Display binaryleri, Windows network helper ve Android build scriptleri kurulum dizinindeki read-only assetlere mutlak yollarla baglanir.
- `download-sources.yml` yalniz ilk materialization sirasinda seed edilir; kullanici degisiklikleri sonraki acilislarda ezilmez.
- Portable paket `README-PORTABLE.txt` marker'i ile AppData materialization'ini bypass eder ve klasor-ici davranisini korur.

## NSIS Dual Install Mode

- `installMode` artik `both` olarak ayarlidir.
- Ayni Setup EXE `/CurrentUser` ile kullaniciya ozel, `/AllUsers` ile system-wide kurulabilir.
- Current-user ve per-machine kurulumlari ayri registry scope'larinda fail-closed dogrulanir.
- Uninstall uygulama dosyalarini kaldirir ancak kullanici runtime/config verisini korur.

## Turkce Installer

- Tauri/NSIS MultiUser sayfasinda upstream Turkish dil dosyasinda eksik kalan bes metin `windows/nsis-hooks.nsh` ile Turkcelestirildi.
- Ingilizce fallback uyarilari build logundan kaldirildi.
- Tauri ana installer template'i fork edilmedi; yalniz desteklenen installer hook mekanizmasi kullanildi.

## CI / Dogrulama

- Windows Distribution kontrati AppData runtime-root, portable marker, dual install-mode ve Turkce MultiUser hook'unu statik olarak kilitler.
- Gercek Windows runner'da NSIS build yapilir.
- Setup gercekten `/CurrentUser` ve `/AllUsers` modlarinda kurulur, Desktop baslatilir, AppData runtime config olusumu kontrol edilir ve iki mod da sessiz uninstall edilir.
- Linux/Windows Compiler Log Gate, Runtime Validation, Host Resilience Policy/Runtime ve artifact-cache gate'leri korunur.

## Uyumluluk

- Breaking change yoktur.
- Config schema 22 korunur.
- Download sources schema 5 korunur.
- Engine API version 24 korunur.
- VM manifest schema 6 korunur.
- Guest catalog schema 4 korunur.
- Android image manifest schema 4 korunur.
- Guest agent protocol version 2 korunur.
- QEMU installer icine gomulu degildir; mevcut TurkuazVM dependency/runtime katmani tarafindan erisilebilir olmalidir.
