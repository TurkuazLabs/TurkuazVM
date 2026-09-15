<!--
Dosya Yolu: /turkuazvm/release/README.md
Amac: TurkuazVM GitHub Release yayin klasorunun kullanimini aciklar
Modul - Markdown
Version: 0.41.3
Aciklama: Release publisher workflow'unun surum metadata kaynaklarini ve idempotent yayin davranisini belgeler
Bagimli Oldugu Katman: Tool | CI/CD
-->

# Release

GitHub Release yayin akisi `.github/workflows/release-publish.yml` tarafindan yonetilir.

Workflow, `Cargo.toml` workspace surumunu okur; ayni surum icin `RELEASE_NOTES_v<surum>.md` ve `RELEASE_STATUS_v<surum>.yml` dosyalarini dogrular. Tag veya GitHub Release zaten varsa tekrar olusturmaz.
