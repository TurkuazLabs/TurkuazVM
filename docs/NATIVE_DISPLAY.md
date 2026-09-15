# 📄 Dosya Yolu: /turkuazvm/docs/NATIVE_DISPLAY.md
# 📌 Amac: TurkuazVM v0.6.0 native VM ekranini kullanma ve sinirlarini aciklar
# 📌 Modul - Markdown
# Version: 0.40.2
# Aciklama: Native display baslatma, dogru RFB/present/render telemetrisi, cached CPU scaling ve gaming fallback sinirini belgeler
# Bagimli Oldugu Katman: Service | Tool | View

# TurkuazDisplay v0.5.0

## Kullanma

1. `config/turkuazvm.yml` icinde `runtime.display_mode: native_rfb` kullanilir.
2. Development akisi icin `scripts/run_desktop.ps1` calistirilir.
3. VM baslatilir.
4. VM kartindaki `Ekran` aksiyonu secilir.
5. Desktop, Engine API v4 ile display session bilgisini alir ve ayri `turkuazvm-display` process'ini acar.

## Kisayollar

- `F1`: mouse capture ac/kapat
- `Escape`: mouse capture serbest birak
- `F11`: fullscreen ac/kapat

## Guvenlik

RFB v0.5.0'da sadece `127.0.0.1` uzerinden acilir. None-auth RFB endpoint remote aga acilmaz.

Remote Engine profili seciliyken `Ekran` acma v0.5.0'da desteklenmez. Remote display daha sonra ayri TLS/streaming transport ile gelecektir.

## Performans telemetrisi

v0.40.2 ile pencere basligindaki degerler ayrildi:

- `RFB UPS`: QEMU RFB serverindan gelen framebuffer update hizi. Sabit ekranda dusuk olmasi normaldir.
- `Mbps`: RAW RFB rectangle payload throughput degeri.
- `Present FPS`: Host pencereye gercek present sayisi. Dirty-frame renderer oldugu icin sabit ekranda dusuk olabilir.
- `Render ms`: `buffer_mut -> scale/copy -> present` yolunun gercek olculen ortalama suresi. `1000 / FPS` turevi degildir.
- `Drop`: Sinirli frame queue renderer gerisinde kaldiginda dusurulen snapshot toplamidir.

v0.40.2 CPU fallback renderer, guest/host boyutu degismedikce nearest-neighbor x/y source maplerini cache eder ve 1:1 boyutta dogrudan framebuffer kopyasi kullanir. `Surface::resize` yalniz render target boyutu degistiginde cagrilir.

## Performans siniri

v0.5.0 renderer Softbuffer tabanli CPU fallback yoludur. Amaci native process, input capture, frame delivery ve display session mimarisini calistirmaktir.

Bu yol PUBG/GameLoop hedefindeki nihai GPU renderer degildir. Gaming yolu daha sonra virtio-gpu/gfxstream/Vulkan tabanli adapterlere gececektir.
