# 📄 Dosya Yolu: /turkuazvm/docs/GAME_PROFILES.md
# 📌 Amac: Game Catalog, compatibility ve profil uygulama akislarini belgeler
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Oyun package eslestirme, runtime gereksinimleri, PUBG Mobile deneysel profili ve Guest Agent entegrasyonunu aciklar
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Game Profiles & Compatibility

TurkuazVM v0.9.0 oyun bilgisini uygulama kodundaki magic stringlerden ayirir. Oyun metadata ve presetleri `config/game-catalog.yml` icinde saklanir ve `turkuazvm-game-catalog` bounded context tarafindan degerlendirilir.

## Dikey Akis

```text
Desktop
  -> Engine API v8
  -> GameCatalogApplicationService
  -> GameCatalogService
  -> GameCatalogRepositoryPort
  <- YamlGameCatalogRepository
```

Compatibility raporu Android device status, Guest Agent capability ve Gaming GPU effective backend bilgilerini tek runtime context icinde degerlendirir.

## Katalog Kaydi

Bir oyun kaydi su tip bilgileri tasir:

- Typed GameId
- Bir veya daha fazla Android package name
- Maturity: experimental / playable / recommended
- Minimum Android SDK
- Kabul edilen ABI listesi
- Persistent multi-touch gereksinimi
- Relative mouse-look gereksinimi
- Tercih edilen GPU backend listesi
- Onerilen width / height / DPI / FPS
- Typed joystick, mouse-look ve input binding presetleri
- Emulator kimliginin acik kalmasi gereksinimi

Ayni game ID veya ayni Android package iki katalog kaydinda kullanilamaz.

## PUBG Mobile Profili

v0.9.0 katalogu PUBG MOBILE icin deneysel bir profil tasir. Bu profil bir uyumluluk/preset kaydidir; oyunun calisacagini garanti etmez.

Profil:

- Android runtime: 1920x1080, 320 DPI, 60 FPS
- Persistent multi-touch gerektirir
- Relative mouse-look gerektirir
- GfxStream veya VirGL/Venus tercih eder
- Keyboard/mouse ve gamepad icin normalized input presetleri saglar
- Emulator disclosure gereksinimini korur

Anti-cheat bypass, emulator detection bypass veya mobile-only matchmaking bypass TurkuazVM kapsaminda degildir.

## Profil Uygulama

`ApplyGameProfile` yalniz VM `Stopped` durumundayken kullanilmalidir. Use-case:

1. Katalog kaydini yukler.
2. Gaming Input profilini typed presetlerden olusturur.
3. Android display/runtime profilini uygular.
4. Android runtime apply basarisiz olursa eski Gaming Input profilini rollback eder.

Bir katalog birden fazla regional package tasiyorsa Gaming Input profili tek bir package'a zorla pinlenmez.

## Compatibility

`GetGameCompatibility` sonucunda:

- status
- package_installed
- blockers
- warnings

ayri alanlar olarak doner.

Ornek blocker:

```text
persistent_multi_touch_required
```

Ornek warning:

```text
gpu_backend_not_preferred:virtio_2d
emulator_identity_must_remain_disclosed
```

## Guest Agent

Persistent multi-touch capability sadece Android Guest Agent gercek handshake sonucu:

```text
available=true
persistent_multi_touch=true
continuation_api=true
```

dondururse READY kabul edilir.

Host tarafinda ADB 5555 yonetim kanali olarak kalir. Guest Agent kendi guest TCP portunu kullanir ve host tarafinda `adb forward` ile loopback endpoint'e tasinir.

## Sinirlar

v0.9.0 halen su katmanlari tamamlamaz:

- Hazir dagitilan Turkuaz Android/AOSP image
- ARM/ARM64 ABI translation
- Windows icin dusuk gecikmeli accelerated Android gaming renderer
- Remote low-latency game streaming
- Oyun bazli otomatik UI element tanima
