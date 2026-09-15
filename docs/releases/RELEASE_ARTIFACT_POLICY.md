# 📄 Dosya Yolu: /turkuazvm/docs/releases/RELEASE_ARTIFACT_POLICY.md
# 📌 Amac: TurkuazVM FULL paketinde aktif ve tarihsel release artefactlarinin saklama politikasini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: FULL pakette yalniz aktif surum kanitlarini ve aktif docs/versions kaydini tutar; tarihsel ozetleri CHANGELOG icinde korur
# Bagimli Oldugu Katman: Tool | View

# Release Artifact Policy

TurkuazVM FULL paket kokunde yalniz aktif surume ait release status ve SHA-256 manifesti tutulur.

Tarihsel surum ozeti `CHANGELOG.md` icinde tutulur. `docs/versions/` FULL paket icinde yalniz aktif surumun ayrintili release notunu tasir; eski faz ve ara surum dokumanlari release paketine kopyalanmaz.

Eski `RELEASE_STATUS_v*.yml`, `MANIFEST_SHA256_v*.txt`, `PATCH_INTEGRATION_*.md`, `VALIDATION_*.md`, cumulative manifest ve H-L gate status dosyalari yeni FULL paketlere kopyalanmaz.

Aktif recovery tabani `docs/recovery/FULL_RECOVERY_STATUS.yml` ile izlenir. Runtime validation artik tarihsel K-L manifest supersession dosyalarina degil aktif config, katman, regression ve current release metadata kontratlarina baglidir.
