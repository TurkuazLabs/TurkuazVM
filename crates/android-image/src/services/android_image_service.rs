// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/services/android_image_service.rs
// # 📌 Amac: Android image define, build-plan, distribution install, register ve VM assignment is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Registry repository, AOSP builder, SDK/CI dagitim portlarini orkestre eder; runtime kind ve Ready assignment kurallarini korur
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::android_image_commands::{
    AssignAndroidImageCommand, CancelAndroidImageDistributionCommand, CleanupAndroidImageDistributionCommand,
    DefineAndroidImageCommand, InstallAndroidImageDistributionCommand, PrepareAndroidImageBuildCommand,
    RegisterAndroidImageBuildCommand,
};
use crate::domain::build_profile::AndroidImageBuildProfile;
use crate::domain::image::{AndroidImage, AndroidImageAssignment, AndroidImageError, AndroidImageId, AndroidImageState};
use crate::ports::android_image_builder_port::{AndroidImageBuildPlan, AndroidImageBuilderError, AndroidImageBuilderPort};
use crate::ports::android_image_distribution_port::{AndroidImageDistributionError, AndroidImageDistributionPort, AndroidImageDistributionProgress};
use crate::ports::android_image_repository_port::{AndroidImageRepositoryError, AndroidImageRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageServiceError {
    Domain(AndroidImageError),
    Repository(AndroidImageRepositoryError),
    Builder(AndroidImageBuilderError),
    Distribution(AndroidImageDistributionError),
    ImageNotReady,
    ImageAlreadyExists,
    ImageAlreadyReady,
    ImageInstallAlreadyRunning,
    ImageInstallNotRunning,
    ImageCleanupNotAllowed,
    AssignmentNotFound,
}

#[derive(Clone)]
pub struct AndroidImageService<
    R: AndroidImageRepositoryPort,
    B: AndroidImageBuilderPort,
    D: AndroidImageDistributionPort,
> {
    repository: R,
    builder: B,
    distribution: D,
}

impl<R, B, D> AndroidImageService<R, B, D>
where
    R: AndroidImageRepositoryPort,
    B: AndroidImageBuilderPort,
    D: AndroidImageDistributionPort,
{
    pub const fn new(repository: R, builder: B, distribution: D) -> Self {
        Self { repository, builder, distribution }
    }

    pub fn list(&self) -> Result<Vec<AndroidImage>, AndroidImageServiceError> {
        self.repository.list().map_err(AndroidImageServiceError::Repository)
    }

    pub fn recover_interrupted_distribution_installs(&self) -> Result<usize, AndroidImageServiceError> {
        let images = self.repository.list().map_err(AndroidImageServiceError::Repository)?;
        let mut recovered = 0_usize;
        for mut image in images {
            if image.state != AndroidImageState::Installing { continue; }
            image.mark_failed(String::from("automatic Android install was interrupted by Engine restart"));
            self.repository.save(image).map_err(AndroidImageServiceError::Repository)?;
            recovered += 1;
        }
        Ok(recovered)
    }

    pub fn get(&self, image_id: &str) -> Result<AndroidImage, AndroidImageServiceError> {
        let id = AndroidImageId::parse(image_id.to_owned()).map_err(AndroidImageServiceError::Domain)?;
        self.repository.get(&id).map_err(AndroidImageServiceError::Repository)
    }

    pub fn distribution_progress(
        &self,
        image_id: &str,
    ) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageServiceError> {
        let image = self.get(image_id)?;
        self.distribution
            .progress(&image)
            .map_err(AndroidImageServiceError::Distribution)
    }

    pub fn define(&self, command: DefineAndroidImageCommand) -> Result<AndroidImage, AndroidImageServiceError> {
        let image_id = AndroidImageId::parse(command.image_id).map_err(AndroidImageServiceError::Domain)?;
        match self.repository.get(&image_id) {
            Ok(_) => return Err(AndroidImageServiceError::ImageAlreadyExists),
            Err(AndroidImageRepositoryError::NotFound(_)) => {}
            Err(error) => return Err(AndroidImageServiceError::Repository(error)),
        }
        let image = AndroidImage::define(image_id, command.name, AndroidImageBuildProfile::gaming_x86_64(), command.requested_release)
            .map_err(AndroidImageServiceError::Domain)?;
        self.repository.save(image.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(image)
    }

    pub fn prepare_build(&self, command: PrepareAndroidImageBuildCommand) -> Result<AndroidImageBuildPlan, AndroidImageServiceError> {
        let mut image = self.get(&command.image_id)?;
        let plan = self.builder.build_plan(&image).map_err(AndroidImageServiceError::Builder)?;
        if plan.supported_host {
            image.mark_build_planned();
            self.repository.save(image).map_err(AndroidImageServiceError::Repository)?;
        }
        Ok(plan)
    }

    pub fn register_build(&self, command: RegisterAndroidImageBuildCommand) -> Result<AndroidImage, AndroidImageServiceError> {
        let mut image = self.get(&command.image_id)?;
        let registration = self.builder.register_build(&image).map_err(AndroidImageServiceError::Builder)?;
        image.register_ready(registration.source_revision, registration.android_release, registration.sdk_level, registration.runtime_kind, registration.artifacts)
            .map_err(AndroidImageServiceError::Domain)?;
        self.repository.save(image.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(image)
    }

    pub fn begin_distribution_install(
        &self,
        command: InstallAndroidImageDistributionCommand,
    ) -> Result<AndroidImage, AndroidImageServiceError> {
        let mut image = self.get(&command.image_id)?;
        if image.state == AndroidImageState::Installing {
            return Err(AndroidImageServiceError::ImageInstallAlreadyRunning);
        }
        if image.state == AndroidImageState::Ready {
            return Err(AndroidImageServiceError::ImageAlreadyReady);
        }
        self.distribution
            .prepare_install(&image)
            .map_err(AndroidImageServiceError::Distribution)?;
        image.mark_installing();
        self.repository.save(image.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(image)
    }

    pub fn cancel_distribution_install(
        &self,
        command: CancelAndroidImageDistributionCommand,
    ) -> Result<AndroidImage, AndroidImageServiceError> {
        let image = self.get(&command.image_id)?;
        if image.state != AndroidImageState::Installing {
            return Err(AndroidImageServiceError::ImageInstallNotRunning);
        }
        self.distribution
            .cancel(&image)
            .map_err(AndroidImageServiceError::Distribution)?;
        Ok(image)
    }

    pub fn cleanup_distribution_install(
        &self,
        command: CleanupAndroidImageDistributionCommand,
    ) -> Result<AndroidImage, AndroidImageServiceError> {
        let mut image = self.get(&command.image_id)?;
        if matches!(image.state, AndroidImageState::Installing | AndroidImageState::Ready) {
            return Err(AndroidImageServiceError::ImageCleanupNotAllowed);
        }
        self.distribution
            .cleanup(&image)
            .map_err(AndroidImageServiceError::Distribution)?;
        image.reset_after_install_cleanup();
        self.repository.save(image.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(image)
    }

    pub fn run_distribution_install(&self, image_id: String) -> Result<AndroidImage, AndroidImageServiceError> {
        let mut image = self.get(&image_id)?;
        match self.distribution.install(&image) {
            Ok(registration) => {
                image.capabilities = registration.capabilities;
                image.register_ready(
                    registration.source_revision,
                    registration.android_release,
                    registration.sdk_level,
                    registration.runtime_kind,
                    registration.artifacts,
                ).map_err(AndroidImageServiceError::Domain)?;
                self.repository.save(image.clone()).map_err(AndroidImageServiceError::Repository)?;
                Ok(image)
            }
            Err(error) => {
                let message = match &error {
                    AndroidImageDistributionError::Cancelled => String::from("Android image installation cancelled by user"),
                    other => format!("{other:?}"),
                };
                image.mark_failed(message);
                self.repository.save(image).map_err(AndroidImageServiceError::Repository)?;
                Err(AndroidImageServiceError::Distribution(error))
            }
        }
    }

    pub fn assign(&self, command: AssignAndroidImageCommand) -> Result<AndroidImageAssignment, AndroidImageServiceError> {
        let image = self.get(&command.image_id)?;
        if image.state != AndroidImageState::Ready { return Err(AndroidImageServiceError::ImageNotReady); }
        let assignment = AndroidImageAssignment::create(command.vm_id, image.id.clone()).map_err(AndroidImageServiceError::Domain)?;
        self.repository.save_assignment(assignment.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(assignment)
    }

    pub fn begin_boot_attempt(&self, vm_id: &str) -> Result<AndroidImageAssignment, AndroidImageServiceError> {
        let mut assignment = self.assignment(vm_id)?.ok_or(AndroidImageServiceError::AssignmentNotFound)?;
        assignment.begin_boot_attempt();
        self.repository.save_assignment(assignment.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(assignment)
    }

    pub fn mark_assignment_ready(&self, vm_id: &str) -> Result<AndroidImageAssignment, AndroidImageServiceError> {
        let mut assignment = self.assignment(vm_id)?.ok_or(AndroidImageServiceError::AssignmentNotFound)?;
        assignment.mark_ready();
        self.repository.save_assignment(assignment.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(assignment)
    }

    pub fn mark_assignment_failed(&self, vm_id: &str, error: String) -> Result<AndroidImageAssignment, AndroidImageServiceError> {
        let mut assignment = self.assignment(vm_id)?.ok_or(AndroidImageServiceError::AssignmentNotFound)?;
        assignment.mark_failed(error);
        self.repository.save_assignment(assignment.clone()).map_err(AndroidImageServiceError::Repository)?;
        Ok(assignment)
    }

    pub fn assignment(&self, vm_id: &str) -> Result<Option<AndroidImageAssignment>, AndroidImageServiceError> {
        self.repository.get_assignment(vm_id).map_err(AndroidImageServiceError::Repository)
    }
}
