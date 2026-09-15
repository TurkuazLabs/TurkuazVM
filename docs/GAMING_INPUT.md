# 📄 Dosya Yolu: /turkuazvm/docs/GAMING_INPUT.md
# 📌 Amac: TurkuazVM Gaming Input kullanimini, Guest Agent ve gamepad davranisini belgeler
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Keymapping editor, normalized koordinat, WASD, mouse-look, persistent multi-touch, ADB fallback ve GilRs adapterini aciklar
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Gaming Input v0.9.0

TurkuazVM Android gaming input ayri `turkuazvm-gaming-input` bounded context olarak modellenir.

## Profil

Her Android gaming VM icin profil:

`data/machines/<vm-id>/gaming-input.yml`

Koordinatlar 0-10000 normalized uzayindadir.

## Runtime Akisi

```text
TurkuazDisplay
  -> Engine API v8
  -> GamingInputApplicationService
  -> GamingInputTranslatorService
  -> AndroidApplicationService
```

F1 mouse capture acikken raw keyboard ve raw mouse motion eventleri local Engine API'ye bounded queue ile aktarilir.

## ADB Fallback

Guest Agent bulunmadiginda ADB fallback su tekil eylemleri uygulayabilir:

- Tap
- Android key event

## Persistent Multi-touch

WASD virtual joystick, mouse-look ve Hold Touch stateful `TouchFrame` uretir. v0.9.0 Engine bu frame'i `AndroidGuestAgentPort` uzerinden guest agent'a gonderir.

Capability ancak agent handshake READY ise aktiftir. Agent yoksa persistent multi-touch sessizce ADB'ye dusurulmez.

Referans guest uygulamasi:

`guest/android-agent`

Bu uygulama Android API 26+ Accessibility gesture continuation mekanizmasini kullanir.

## Gamepad

Concrete adapter:

`apps/display/src/tools/gamepad_input_tool.rs`

Adapter GilRs eventlerini TurkuazVM normalize button/axis kimliklerine cevirir. Gamepad eventleri Tauri WebView uzerinden gecmez; TurkuazDisplay local input bridge'i kullanir.

Normalized ana button ID'leri:

```text
0 South
1 East
2 North
3 West
```

Left/right stick axis ID'leri:

```text
0 Left X
1 Left Y
2 Right X
3 Right Y
```

## Guvenlik

- Native Gaming Input bridge yalniz local Engine endpointinde aktiftir.
- Remote display v0.9.0 kapsaminda degildir.
- Emulator kimligi veya anti-cheat bypass bu katmanin parcasi degildir.
- Guest Agent guest tarafinda loopback portunda dinler; host erisimi ADB forward uzerinden saglanir.
