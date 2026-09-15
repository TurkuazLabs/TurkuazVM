# 📄 Dosya Yolu: /turkuazvm/docs/architecture/qmp-session.md
# 📌 Amac: TurkuazVM QMP client session akis ve hata kurallarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: QMP greeting, capability negotiation, correlation ve event ayiklama modelini belgeler
# Bagimli Oldugu Katman: Service | Tool

# QMP Session

## Akis

1. TCP transport QEMU QMP endpointine baglanir.
2. Client ilk mesaj olarak QMP greeting bekler.
3. Client qmp_capabilities komutunu request id ile gonderir.
4. Basarili negotiation sonrasinda query-version gonderilir.
5. query-commands ile runtime command listesi alinir.
6. Her response request id ile eslestirilir.
7. Event mesajlari response correlation akisindan ayrilir.

## Sinirlar

Core crate QMP JSON alan adlarini veya command isimlerini bilmez.
QMP command sabitleri qemu crate icinde tutulur.
TCP sadece transport adapteridir; protocol client TCP'ye bagimli degildir.
Testler FakeTransport ile gercek QEMU baslatmadan handshake ve error mapping dogrular.

## Guvenlik

QMP endpoint varsayilan olarak uzak aga acilmayacak.
VM process lifecycle eklendiginde endpoint local host veya local IPC ile sinirlanacak.
Remote host modu ayrica kimlik dogrulama ve sifreli transport gerektirecek.
