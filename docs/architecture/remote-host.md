# 📄 Dosya Yolu: /turkuazvm/docs/architecture/remote-host.md
# 📌 Amac: TurkuazVM v0.6.0 local/remote Engine management mimarisini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Host profile, TLS transport, token auth ve headless Engine sinirlarini belgeler
# Bagimli Oldugu Katman: Controller | Service | Tool | View

# Remote Host Architecture

## Ana Akis

```text
TurkuazVM Desktop
  -> Tauri Controller
  -> DesktopService
  -> Active Host Profile
  -> EngineApiClientTool
  -> Plain loopback veya TLS TCP
  -> EngineApiServerTool
  -> EngineApplicationService
  -> EngineAuthService
  -> Core use-case
```

Desktop QEMU veya host network adapterlerini remote durumda da tanimaz. Endpoint degisikligi yalniz DesktopService tarafindaki aktif host profilini degistirir.

## Local Profil

```text
Desktop
  -> 127.0.0.1
  -> plain loopback
  -> optional auth
  -> local Engine auto-start
```

Local profil loopback disi IP kullanamaz.

## Remote Profil

```text
Windows Desktop
  -> TLS + token
  -> LAN/VPN
  -> Linux Headless Engine
  -> QEMU + KVM
```

Remote Desktop profili TLS veya token olmadan config validation'i gecemez.

## Engine Bind Kurali

Engine API loopback disina bind olacaksa TLS ve token birlikte zorunludur. QMP ise her zaman loopback adresinde kalir. Bu kural transport konfigurasyon hatasini runtime'a birakmadan startup asamasinda reddeder.

## TLS Trust

Engine server certificate ve private key kullanir. Desktop ise CA certificate ve `server_name` ile peer dogrulamasi yapar. TLS kodu `turkuazvm-transport` crate icinde kalir; Core ve Engine API contract crate rustls bilmez.

## Authentication

Token EngineRequest icinde yalniz transport payload olarak bulunur. EngineApplicationService action dispatch oncesi EngineAuthService ile dogrulama yapar. Token Dashboard veya error output icinde geri donmez.

## Secret Ownership

Token ve private key `secrets/` altinda tutulur ve Git'e girmez. Production deployment secret dosyalarini OS permission ve secret management politikasi ile korumalidir.

## Headless Engine

Ayni `turkuazvm-engine` binary hem local hem remote kullanilir. Remote Linux deployment display mode `none` ile calisabilir. Ayrica fork edilmis server edition yoktur; Domain ve Application katmanlari tek kaynak olarak kalir.

## Sonraki Genisleme

v0.5.0 Native Display/Input ile VM display stream'i Engine management API ile ayni kanala zorlanmayacak. Management plane ve display/data plane ayri port/adapter sinirlari olarak tasarlanacak.
