# 📄 Dosya Yolu: /turkuazvm/docs/architecture/game-profiles.md
# 📌 Amac: Game Profiles, Guest Agent ve gamepad adapter mimari sinirlarini tanimlar
# 📌 Modul - Markdown
# Version: 0.28.0
# Aciklama: Game Catalog bounded context, compatibility context, persistent touch protocol ve host gamepad adapterini Hexagonal Architecture icinde konumlandirir
# Bagimli Oldugu Katman: Service | Repo | Tool | View

# Game Profiles Architecture v0.9.0

## Bounded Context

```text
Game Catalog Domain
  GameDefinition
  GameId
  PackageName
  CatalogKey
  CatalogMouseButton
  GameRequirements
  RecommendedAndroidRuntime

Application
  GameCatalogApplicationService

Port
  GameCatalogRepositoryPort

Adapter
  YamlGameCatalogRepository
```

Game Catalog su katmanlari bilmez:

- QEMU
- Android ADB implementation
- Gaming Input crate
- Engine API
- Tauri
- serde/YAML

YAML string representation yalniz Repository DTO sinirindadir. Domain key/mouse degerleri typed enum olarak tutulur.

## Compatibility Context

```text
Android status ----+
Guest Agent --------+--> GameRuntimeContext --> GameCatalogService.evaluate
Gaming GPU ---------+
```

Compatibility sonucu runtime capability ile katalog maturity bilgisini ayri tutar. Bir blocker varsa sonuc `Blocked`; blocker yoksa katalog maturity seviyesi korunur.

## Persistent Multi-touch

```text
TurkuazDisplay
  -> Engine API v8
  -> GamingInputTranslatorService
  -> TouchFrame
  -> AndroidApplicationService
  -> AndroidGuestAgentPort
  <- AndroidGuestAgentTool
  -> adb forward
  -> Turkuaz Input Agent
  -> AccessibilityService GestureDescription
```

Wire contract ayri `turkuazvm-guest-agent-protocol` crate'indedir. Protocol version 1 newline JSON kullanir.

Host ve guest implementasyonlari protocol crate'in semantic contractina baglidir; Gaming Input domain TCP/JSON/ADB bilmez.

## Gamepad

```text
GilRs
  -> GamepadInputTool
  implements HostGamepadPort
  -> GamingInputEvent
  -> GamingInputBridgeTool
  -> Engine API v8
```

GilRs button/axis kimlikleri Domain'e dogrudan sizdirilmaz. `GamepadInputTool` bunlari TurkuazVM normalize button/axis ID'lerine cevirir.

## Transaction

Game profile apply sirasinda:

```text
old input profile
  -> save new input profile
  -> configure Android runtime
       success -> commit
       failure -> restore/delete input profile
```

Controller transaction mantigi tutmaz; orchestration Engine Application Service katmanindadir.
