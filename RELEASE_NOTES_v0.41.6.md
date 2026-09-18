# 📄 Dosya Yolu: /turkuazvm/RELEASE_NOTES_v0.41.6.md
# 📌 Amac: TurkuazVM v0.41.6 GitHub release notlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.41.6
# Aciklama: Windows kurulum kokunu TurkuazLabs hiyerarsisine, buyuk VM/ISO/image verilerini USERPROFILE/TurkuazVM kokune alan patchi ozetler
# Bagimli Oldugu Katman: Tool | View | CI/CD

# TurkuazVM v0.41.6

v0.41.6, Windows installer icin marka klasor hiyerarsisini duzeltir. Kurulu uygulama dosyalari artik dogrudan `TurkuazVM` klasoru yerine `TurkuazLabs/TurkuazVM` altinda tutulur.

## Windows Install Root

- System-wide kurulum varsayilani: `C:\Program Files\TurkuazLabs\TurkuazVM`.
- Current-user kurulum varsayilani: `%LOCALAPPDATA%\TurkuazLabs\TurkuazVM`.
- Tek Setup EXE icindeki `/CurrentUser` ve `/AllUsers` modlari korunur.
- Registry scope ve uninstall davranisi degismez.

## Runtime ve Kullanici Verisi

- Writable config/runtime koku `%LOCALAPPDATA%\TurkuazVM` olarak kalir.
- Buyuk kullanici verileri varsayilan olarak `%USERPROFILE%\TurkuazVM` altinda toplanir.
- Yeni qcow2/raw diskler ve VM'ye baglanan installer ISO kopyalari: `%USERPROFILE%\TurkuazVM\VMs\<vm-id>\...`.
- Indirilen installer ISO'lari: `%USERPROFILE%\TurkuazVM\ISOs`.
- Android virtual/system image ciktilari: `%USERPROFILE%\TurkuazVM\Images\Android`.\n- VM-ozel Android private disk ve AVD userdata: `%USERPROFILE%\TurkuazVM\VMs\<vm-id>\runtime\...`.
- Test ve ileri seviye kurulumlar icin `TURKUAZVM_USER_DATA_ROOT` override'i desteklenir; normal kullanici icin ayar gerektirmez.
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
- AppData config/runtime materialization, USERPROFILE/TurkuazVM VMs/ISOs/Images kokleri ve uninstall sonrasi kullanici verisinin korunmasi birlikte test edilir.

## Uyumluluk

- Breaking change yoktur.
- Config schema 22 korunur.
- Download sources schema 5 korunur.
- Engine API version 24 korunur.
- VM manifest schema 6 korunur.
- Guest catalog schema 4 korunur.
- Android image manifest schema 4 korunur.
- Guest agent protocol version 2 korunur.
- v0.41.5 ve daha eski kurulu VM metadata'si `%LOCALAPPDATA%\TurkuazVM` altinda kalir.
- Eski `data/machines/<vm>/disks`, bagli ISO ve Android runtime media konumlari legacy fallback ile okunur; otomatik zorunlu veri tasimasi yapilmaz.
- Mevcut kullaniciya ozel `download-sources.yml` yollarina dokunulmaz; yeni kurulumlar USERPROFILE koklerini kullanir.
- QEMU installer icine gomulu degildir; mevcut dependency/runtime katmani tarafindan erisilebilir olmalidir.
