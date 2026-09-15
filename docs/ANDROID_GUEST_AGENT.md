# 📄 Dosya Yolu: /turkuazvm/docs/ANDROID_GUEST_AGENT.md
# 📌 Amac: Turkuaz Android Guest Agent kurulum, protocol ve runtime sinirlarini belgeler
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: AOSP referans AccessibilityService agent, ADB forward ve persistent multi-touch akisinin nasil calistigini aciklar
# Bagimli Oldugu Katman: Service | Tool | Config

# Android Guest Agent v0.11.8

Turkuaz Input Agent persistent joystick, mouse-look ve HoldTouch frame'lerini Android guest icinde uygulayan referans sistem uygulamasidir.

Kaynak:

`guest/android-agent`

## Mimari

```text
TurkuazDisplay
  -> Engine API v8
  -> GamingInputTranslatorService
  -> AndroidGuestAgentPort
  <- AndroidGuestAgentTool
  -> adb forward
  -> guest 127.0.0.1:5566
  -> TurkuazInputAgent
  -> AccessibilityService
  -> GestureDescription
```

ADB 5555 yonetim kanali olarak kalir. Guest Agent ayri guest portu kullanir. Host tarafindaki forward portu `android.guest_agent.host_port_min/max` araligindan runtime'da ayrilir.

## Protocol

Wire contract:

`crates/guest-agent-protocol`

Protocol version 1 newline-delimited JSON kullanir.

Desteklenen action'lar:

- Ping
- Capabilities
- ApplyTouchFrame
- ResetInput

Capability ancak gercek handshake cevabi alindiginda READY kabul edilir.

## AOSP Entegrasyonu

`guest/android-agent/Android.bp` referans APK'yi AOSP Soong build'ine kaydeder.

TurkuazInputAgent AOSP product image icine dahil edilir. v0.11.8 first-boot akisi APK varligini `pm path` ile dogrular, mevcut `enabled_accessibility_services` listesini korur, Turkuaz service componentini listeye ekler, `accessibility_enabled=1` uygular ve ardindan Guest Agent Ping/Pong readiness bekler.

## Input Modeli

Host her degisimde full active touch frame gonderir. Agent pointer ID bazinda aktif stroke state'i tutar.

- Down yeni devam edebilir stroke baslatir.
- Move onceki stroke'u continuation ile ilerletir.
- Up continuation zincirini sonlandirir.
- Reset aktif pointer'lari kontrollu olarak kapatir.

Agent normalized 0-10000 koordinatlarini guest ekran pixel boyutuna cevirir.

## Guvenlik

- Guest server yalniz loopback adresinde dinler.
- Host erisimi ADB forward ile saglanir.
- Protocol version ve request correlation host tarafinda kontrol edilir.
- Emulator kimligi veya anti-cheat bypass bu agent'in parcasi degildir.

## v0.11.8 Siniri

Bu agent persistent multi-touch ve first-boot readiness zincirini saglar. Windows accelerated Android gaming renderer ve ARM/ARM64 compatibility/translation halen sonraki kalite kapilaridir.
