// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/services/android_input_service.rs
// # 📌 Amac: Android guest input injection use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Typed input action validation, profile lookup ve input tool adapterini orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::android_input_command::AndroidInputCommand;
use crate::domain::input::AndroidInputError;
use crate::domain::runtime_profile::{AndroidProfileError, AndroidVmId};
use crate::ports::android_input_port::{AndroidInputPort, AndroidInputPortError};
use crate::ports::android_profile_repository_port::{
    AndroidProfileRepositoryError, AndroidProfileRepositoryPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidInputServiceError {
    ProfileDomain(AndroidProfileError),
    InputDomain(AndroidInputError),
    Repository(AndroidProfileRepositoryError),
    InputPort(AndroidInputPortError),
}

pub struct AndroidInputService<R, I>
where
    R: AndroidProfileRepositoryPort,
    I: AndroidInputPort,
{
    repository: R,
    input: I,
}

impl<R, I> AndroidInputService<R, I>
where
    R: AndroidProfileRepositoryPort,
    I: AndroidInputPort,
{
    pub const fn new(repository: R, input: I) -> Self {
        Self { repository, input }
    }

    pub fn inject(&self, command: AndroidInputCommand) -> Result<(), AndroidInputServiceError> {
        let vm_id = AndroidVmId::parse(command.vm_id)
            .map_err(AndroidInputServiceError::ProfileDomain)?;
        let profile = self
            .repository
            .get(&vm_id)
            .map_err(AndroidInputServiceError::Repository)?;
        let action = command
            .action
            .validate()
            .map_err(AndroidInputServiceError::InputDomain)?;
        action
            .validate_bounds(profile.display().width(), profile.display().height())
            .map_err(AndroidInputServiceError::InputDomain)?;
        self.input
            .inject(&profile, &action)
            .map_err(AndroidInputServiceError::InputPort)
    }
}
