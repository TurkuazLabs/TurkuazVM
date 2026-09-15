// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/gaming_input_application_service.rs
// # 📌 Amac: Gaming Input bounded context ile VM bounded contextini Engine seviyesinde orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Android VM preflight, keymapping profile persistence, capability ve stateful raw input translation use-case'lerini birlestirir
// # Bagimli Oldugu Katman: Service | Repo

use turkuazvm_core::domain::guest_boot::GuestProfile;
use turkuazvm_core::services::vm_query_service::VmQueryService;
use turkuazvm_gaming_input::domain::capability::GamingInputCapabilities;
use turkuazvm_gaming_input::domain::event::GamingInputEvent;
use turkuazvm_gaming_input::domain::plan::GamingInputPlan;
use turkuazvm_gaming_input::domain::profile::GameInputProfile;
use turkuazvm_gaming_input::services::game_input_profile_service::GameInputProfileService;
use turkuazvm_gaming_input::services::gaming_input_translator_service::GamingInputTranslatorService;
use turkuazvm_repositories::repositories::shared_vm_repository::SharedVmRepository;
use turkuazvm_repositories::repositories::yaml_game_input_profile_repository::YamlGameInputProfileRepository;
use turkuazvm_repositories::repositories::yaml_vm_repository::YamlVmRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingInputApplicationError {
    Vm(String),
    VmProfileMustBeAndroid,
    Profile(String),
}

type EngineRepository = SharedVmRepository<YamlVmRepository>;
type ProfileService = GameInputProfileService<YamlGameInputProfileRepository>;

pub struct GamingInputApplicationService {
    query_service: VmQueryService<EngineRepository>,
    profile_service: ProfileService,
    translator: GamingInputTranslatorService,
}

impl GamingInputApplicationService {
    pub fn new(repository: EngineRepository, data_root: std::path::PathBuf) -> Self {
        Self {
            query_service: VmQueryService::new(repository),
            profile_service: GameInputProfileService::new(YamlGameInputProfileRepository::new(data_root)),
            translator: GamingInputTranslatorService::default(),
        }
    }

    pub const fn capabilities(&self) -> GamingInputCapabilities {
        GamingInputCapabilities::foundation()
    }

    pub fn capabilities_for(&self, vm_id: &str) -> Result<GamingInputCapabilities, GamingInputApplicationError> {
        self.require_android_vm(vm_id)?;
        Ok(self.capabilities())
    }

    pub fn save_profile(&mut self, profile: GameInputProfile) -> Result<GameInputProfile, GamingInputApplicationError> {
        self.require_android_vm(profile.vm_id().as_str())?;
        self.profile_service
            .save(profile)
            .map_err(|error| GamingInputApplicationError::Profile(format!("{error:?}")))
    }

    pub fn delete_profile(&mut self, vm_id: &str) -> Result<(), GamingInputApplicationError> {
        self.require_android_vm(vm_id)?;
        self.profile_service
            .delete(vm_id)
            .map_err(|error| GamingInputApplicationError::Profile(format!("{error:?}")))
    }

    pub fn profile(&self, vm_id: &str) -> Result<GameInputProfile, GamingInputApplicationError> {
        self.require_android_vm(vm_id)?;
        self.profile_service
            .get(vm_id)
            .map_err(|error| GamingInputApplicationError::Profile(format!("{error:?}")))
    }

    pub fn translate(
        &mut self,
        vm_id: &str,
        event: GamingInputEvent,
    ) -> Result<GamingInputPlan, GamingInputApplicationError> {
        let profile = self.profile(vm_id)?;
        Ok(self.translator.translate(&profile, event))
    }

    pub fn reset_state(&mut self, vm_id: &str) -> Result<(), GamingInputApplicationError> {
        self.require_android_vm(vm_id)?;
        self.translator.reset(vm_id);
        Ok(())
    }

    fn require_android_vm(&self, vm_id: &str) -> Result<(), GamingInputApplicationError> {
        let machine = self
            .query_service
            .get(vm_id)
            .map_err(|error| GamingInputApplicationError::Vm(format!("{error:?}")))?;
        if machine.guest_boot().profile() != GuestProfile::Android {
            return Err(GamingInputApplicationError::VmProfileMustBeAndroid);
        }
        Ok(())
    }
}
