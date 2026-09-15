# 📄 Dosya Yolu: /turkuazvm/docs/GAMING_GPU.md
# 📌 Amac: TurkuazVM Gaming GPU capability ve backend secim davranisini aciklar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: virtio-gpu, VirGL/Venus, rutabaga/GfxStream ve fallback politikalarini belgeler
# Bagimli Oldugu Katman: Service | Tool | View

# Gaming GPU Foundation

TurkuazVM v0.7.0 GPU katmanini host capability, hypervisor capability ve policy olarak uc parcaya ayirir.

## Backendler

- `software`: QEMU varsayilan software/display yolu.
- `virtio_2d`: Paravirtualized 2D virtio-gpu.
- `virgl_venus`: Linux host icin VirGL + blob + Venus Vulkan yolu.
- `gfxstream`: Linux host icin rutabaga/GfxStream experimental Android yolu.

## Auto Policy

`gpu.mode: auto` su sirayi uygular:

1. Experimental opt-in aciksa ve Linux + Vulkan + QEMU rutabaga/GfxStream Android capsetleri varsa `gfxstream`.
2. Linux + Vulkan + QEMU VirGL + Venus varsa `virgl_venus`.
3. QEMU virtio-gpu varsa `virtio_2d`.
4. Aksi halde `software`.

Experimental GfxStream explicit opt-in olmadan secilmez.

## Windows Siniri

v0.7.0 stock QEMU icin Windows GPU acceleration'i destekleniyor kabul etmez. Windows hostta capability raporu uretilir ancak accelerated backend secimi Linux host kosullari ile sinirlidir.

PUBG/GameLoop hedefi icin Windows tarafinda sonraki surumlerde ayri renderer/runtime adapteri gerekir. Bu adapter mevcut GPU domain ve Engine API capability modellerini koruyacak.

## Config

```text
gpu:
  mode: auto
  hostmem_mib: 1024
  allow_experimental_android_gfxstream: false
```

Desteklenen mode degerleri:

- `auto`
- `software`
- `virtio_2d`
- `virgl_venus`
- `gfxstream`

`hostmem_mib` 256-8192 MiB araliginda olmalidir.

## Telemetry

TurkuazDisplay pencere basliginda anlik FPS ve ortalama frametime gosterir. Bu v0.7.0 telemetry foundation'dir; GPU utilization, render queue ve frame pacing histogrami sonraki performans surumlerinde eklenecektir.
