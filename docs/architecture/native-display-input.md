# 📄 Dosya Yolu: /turkuazvm/docs/architecture/native-display-input.md
# 📌 Amac: TurkuazVM native display ve input katmaninin process, transport ve dependency sinirlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.40.2
# Aciklama: Local RFB framebuffer fallback, atomik transport telemetrisi, cached CPU scaling, native input ve gelecekteki gaming renderer gecis noktasini belgeler
# Bagimli Oldugu Katman: Controller | Service | Tool | View

# Native Display ve Input Architecture

## Hedef

v0.5.0 ile VM goruntusu Tauri WebView icinde render edilmez. Tauri yalniz yonetim UI'sidir. Gercek VM penceresi ayri `turkuazvm-display` process'i tarafindan olusturulur.

```text
TurkuazVM Desktop
    -> DesktopController
    -> DesktopService
    -> Engine API v4
    -> GetDisplaySession
    -> DisplayProcessTool
    -> turkuazvm-display
        -> DisplayController
        -> DisplayService
        -> RfbClientTool
        -> NativeDisplayView
```

## QEMU display akisi

```text
QemuRuntimeTool
    -> DisplayRuntimeInfo
    -> QemuDisplayRuntimePlan
    -> QemuCommandBuilder
    -> -display none
    -> -vnc 127.0.0.1:N,share=force-shared
    -> TCP 5900+N
    -> TurkuazDisplay RFB client
```

RFB endpoint v0.5.0'da yalniz `127.0.0.1` uzerindedir. Engine API remote olsa bile framebuffer endpoint remote aga acilmaz.

## Domain siniri

Core yalniz su protocol-neutral kavramlari bilir:

- `DisplayRuntimeInfo`
- `DisplayTransport`
- `DisplayCapabilities`
- `InputCaptureMode`

Core `-vnc`, RFB wire format, Winit veya Softbuffer bilmez.

## RFB adapter sorumlulugu

`RfbClientTool`:

- RFB 3.3, 3.7 ve 3.8 handshake uygular.
- None security yalniz loopback endpoint'te kabul edilir.
- 32-bit true-color pixel format ister.
- Raw framebuffer encoding kullanir.
- DesktopSize pseudo-encoding ile resolution degisimini kabul eder.
- Keyboard ve absolute pointer mesajlarini server'a gonderir.
- Frame queue derinligini sinirli tutar; renderer geride kalirsa eski frame biriktirmez.

## Native window sorumlulugu

`NativeDisplayView`:

- Winit ile native window olusturur.
- Softbuffer ile CPU fallback framebuffer present eder.
- F11 ile fullscreen degistirir.
- F1 ile mouse capture acar/kapatir.
- Escape capture'i serbest birakir.
- Captured mouse `DeviceEvent::MouseMotion` delta bilgisini alir.
- Keyboard fiziksel key code bilgisini RFB keysym'e cevirir.
- RFB UPS, RAW Mbps, present FPS ve gercek render duration telemetrisini `FrameClockTool` ile ayri olcer.
- RFB reader update/rectangle/raw-byte/enqueue/drop sayaçlarini atomik tutar.
- Nearest-neighbor x/y eslemelerini `ScaleMapTool` ile boyut degisene kadar cache eder.
- 1:1 guest/host boyutunda per-pixel scaling yerine dogrudan framebuffer kopyasi yapar.

## Relative mouse siniri

v0.5.0 host tarafinda relative mouse motion yakalayabilir. Ancak RFB pointer protokolu guest'e absolute koordinat gonderir. Bu nedenle `relative_pointer_capture` capability host capture ozelligini ifade eder; gercek guest-relative injection degildir.

Gaming runtime icin sonraki input adapteri QMP/virtio/Android touch katmanina gecerek bu siniri degistirebilir. Domain ve Desktop API degismek zorunda kalmaz.

## Gamepad siniri

Gamepad capability domain ve Engine API'de ayrilmistir ancak v0.5.0'da gercek gamepad observation/injection adapteri yoktur. Bu nedenle capability `false` doner.

## Remote host siniri

v0.5.0 Desktop, `mode: remote` host icin native display acmayi reddeder. Bunun nedeni Engine tarafindaki RFB endpoint'in bilincli olarak host-loopback kalmasidir.

Remote framebuffer icin ileride ayri authenticated/TLS streaming adapteri gereklidir. QMP remote aga acilmayacaktir.

## Gaming'e gecis

RFB + Softbuffer yolu ilk native display foundation ve compatibility fallback'tir. Nihai gaming render yolu degildir.

Gelecekte:

```text
Android Guest
    -> virtio-gpu / gfxstream
    -> host GPU
    -> TurkuazDisplay GPU surface
```

gecisinde `DisplayRuntimeInfo` ve Desktop `GetDisplaySession` siniri korunur; yalniz display transport/renderer adapterleri degisir.
