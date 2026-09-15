// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/controllers/engine_api_controller.rs
// # 📌 Amac: Local Engine API requestini application service'e yonlendirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Request kabul eder ve is mantigi eklemeden EngineApplicationService cagirir
// # Bagimli Oldugu Katman: Service

use turkuazvm_engine_api::{EngineRequest, EngineResponse};

use crate::services::engine_application_service::EngineApplicationService;

pub struct EngineApiController {
    service: EngineApplicationService,
}

impl EngineApiController {
    pub const fn new(service: EngineApplicationService) -> Self {
        Self { service }
    }

    pub fn maintenance_tick(&mut self) -> Result<(), String> {
        self.service.maintenance_tick()
    }

    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        self.service.handle(request)
    }
}
