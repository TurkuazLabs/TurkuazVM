# 📄 Dosya Yolu: /turkuazvm/docs/architecture/gaming-input.md
# 📌 Amac: Gaming Input bounded context katman ve veri akislarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Profile persistence, stateful translator, native input bridge ve Android sink sinirlarini mimari olarak kilitler
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Gaming Input Architecture

```text
TurkuazDisplay
  -> GamingInputBridgeTool
  -> Engine API v8
  -> EngineApplicationService
  -> GamingInputApplicationService
     -> GameInputProfileService
        -> GameInputProfileRepositoryPort
           <- YamlGameInputProfileRepository
     -> GamingInputTranslatorService
        -> GamingInputPlan
  -> AndroidApplicationService
     -> ADB single-touch fallback
     -> Guest Agent persistent multi-touch
```

## Domain

`crates/gaming-input` dis sistem bilmez. QEMU, ADB, Tauri, serde/YAML ve platform process API'leri bu crate icine giremez.

## Pointer Modeli

- Pointer 0: virtual joystick
- Pointer 1: mouse look
- Binding HoldTouch pointer kimlikleri bu iki reserved kimlikle cakismayacak sekilde profile tasarlanmalidir.

v0.9.0 profile domain joystick, mouse-look ve HoldTouch pointer ID cakismalarini engeller.

## Stateful Translator

W/A/S/D ayni anda basili olabilir. Translator VM basina pressed-key state tutar ve diagonal hareketi tek joystick touch position planina cevirir.

Mouse look sadece capture aktifken calisir. Capture kapandiginda aktif mouse-look pointer icin `Up` planlanir.

## Transport

Editor islemleri normal Desktop -> Engine API yolundadir. Oyun anindaki native eventler Tauri WebView'a geri donmez; TurkuazDisplay local Engine API'ye dogrudan iletir.
