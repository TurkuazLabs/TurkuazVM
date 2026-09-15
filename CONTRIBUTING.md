# 📄 Dosya Yolu: /turkuazvm/CONTRIBUTING.md
# 📌 Amac: TurkuazVM gelistirme, branch, PR ve kalite kurallarini tanimlar
# 📌 Modul - Markdown
# Version: 1.0.0
# Aciklama: Katki akisinda layered architecture, regression testi ve CI gate zorunluluklarini standartlastirir
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View | Config

# TurkuazVM Katki Rehberi

## Branch Akisi

`main` release-ready kaynak dalidir. Gelistirme degisiklikleri ayri branch uzerinde yapilir ve Pull Request ile `main` dalina alinir.

Onerilen branch onekleri:

- `fix/` bug ve hotfix
- `feat/` geriye uyumlu yeni ozellik
- `chore/` CI, dokumantasyon ve bakim
- `release/` release hazirligi

## Mimari

Zorunlu akis: Controller -> Service -> Repo/Model -> Tool -> View -> Language/Config.

Controller request alir ve Service cagirir. Is kurali Service katmanindadir. DB ve storage erisimi Repo/Model katmanindadir. Dis dunya adaptorleri Tool katmanindadir. View output ve UI katmanidir. Magic string ve inline config kullanilmaz.

## Degisiklik Kurallari

Her davranis degisikligi regression testi veya contract ile korunur. Mevcut fail-closed gate davranisi gevsetilmez. Warning veya lint bastirma, gercek hatayi saklamak icin kullanilmaz.

## CI

Bir PR merge edilmeden once asagidaki kontroller PASS olmalidir:

- TurkuazVM Compiler Log Gate
- TurkuazVM Runtime Validation Gate
- TurkuazVM Host Resilience Policy Gate
- TurkuazVM Host Resilience Runtime Gate
- artifact-cache-agent-gate

## Surumleme

Patch bug veya hotfix icindir. Minor yeni modul veya geriye uyumlu ozelliktir. Major mimari veya breaking change icindir. Release metadata ve uygulama surumleri ayni release numarasinda tutulur.
