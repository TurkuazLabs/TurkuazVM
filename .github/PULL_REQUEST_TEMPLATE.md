# 📄 Dosya Yolu: /turkuazvm/.github/PULL_REQUEST_TEMPLATE.md
# 📌 Amac: TurkuazVM pull request inceleme ve kalite kontrol sablonunu tanimlar
# 📌 Modul - Markdown
# Version: 1.0.0
# Aciklama: Mimari, test, surumleme ve CI kontrollerini her PR icin standart hale getirir
# Bagimli Oldugu Katman: Config | View

# Degisiklik Ozeti

Bu PR neyi degistiriyor ve neden gerekli?

# Mimari Kontrol

- [ ] Controller yalniz request aliyor ve Service cagiriyor.
- [ ] Is kurali Service katmaninda.
- [ ] DB veya storage islemleri Repo/Model katmaninda.
- [ ] Dis dunya entegrasyonlari Tool katmaninda.
- [ ] View yalniz output ve UI sorumlulugu tasiyor.
- [ ] Magic string veya inline config eklenmedi.

# Dogrulama

- [ ] TurkuazVM Compiler Log Gate PASS.
- [ ] TurkuazVM Runtime Validation Gate PASS.
- [ ] TurkuazVM Host Resilience Policy Gate PASS.
- [ ] TurkuazVM Host Resilience Runtime Gate PASS.
- [ ] artifact-cache-agent-gate PASS.
- [ ] Yeni davranis icin regression testi veya contract eklendi.

# Surumleme

- [ ] Patch: bug veya hotfix.
- [ ] Minor: yeni modul veya geriye uyumlu ozellik.
- [ ] Major: mimari veya breaking change.
- [ ] Release metadata gerekiyorsa guncellendi.

# Risk ve Geri Donus

Risk, migration, rollback veya recovery notlarini buraya yazin.
