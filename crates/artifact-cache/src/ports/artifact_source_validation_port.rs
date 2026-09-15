// # 📄 Dosya Yolu: /turkuazvm/crates/artifact-cache/src/ports/artifact_source_validation_port.rs
// # 📌 Amac: Mutable HTTP artifact kaynaklarinin ETag/Last-Modified revalidation contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Cache Service'i curl/HTTP implementasyonundan ayirir ve conditional source validation sonucunu typed tasir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::artifact_cache::ArtifactSourceValidators;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactSourceValidationError {
    Transient(String),
    Policy(String),
    InvalidResponse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactSourceValidationOutcome {
    NotModified { validators: ArtifactSourceValidators },
    Modified { validators: ArtifactSourceValidators },
}

pub trait ArtifactSourceValidationPort: Send {
    fn revalidate(
        &mut self,
        source_url: &str,
        validators: &ArtifactSourceValidators,
    ) -> Result<ArtifactSourceValidationOutcome, ArtifactSourceValidationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopArtifactSourceValidationPort;

impl ArtifactSourceValidationPort for NoopArtifactSourceValidationPort {
    fn revalidate(
        &mut self,
        _source_url: &str,
        _validators: &ArtifactSourceValidators,
    ) -> Result<ArtifactSourceValidationOutcome, ArtifactSourceValidationError> {
        Err(ArtifactSourceValidationError::Transient(String::from(
            "artifact source validation adapter is unavailable",
        )))
    }
}
