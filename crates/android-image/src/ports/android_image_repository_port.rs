// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_image_repository_port.rs
// # 📌 Amac: Android image registry ve VM assignment persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: YAML/file veya gelecekte remote registry adapterlerini Service katmanindan ayirir
// # Bagimli Oldugu Katman: Repo

use crate::domain::image::{AndroidImage, AndroidImageAssignment, AndroidImageId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageRepositoryError {
    NotFound(AndroidImageId),
    Storage(String),
}

pub trait AndroidImageRepositoryPort {
    fn list(&self) -> Result<Vec<AndroidImage>, AndroidImageRepositoryError>;
    fn get(&self, image_id: &AndroidImageId) -> Result<AndroidImage, AndroidImageRepositoryError>;
    fn save(&self, image: AndroidImage) -> Result<(), AndroidImageRepositoryError>;
    fn get_assignment(&self, vm_id: &str) -> Result<Option<AndroidImageAssignment>, AndroidImageRepositoryError>;
    fn save_assignment(&self, assignment: AndroidImageAssignment) -> Result<(), AndroidImageRepositoryError>;
}
