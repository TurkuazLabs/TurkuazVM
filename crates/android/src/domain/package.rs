// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/domain/package.rs
// # 📌 Amac: Android package kimligi ve package metadata invariantlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: APK/package islemlerini serbest string yerine typed modellerle tasir
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AndroidPackageName(String);

impl AndroidPackageName {
    pub fn parse(value: impl Into<String>) -> Result<Self, AndroidPackageError> {
        let value = value.into();
        let valid = value.len() >= 3
            && value.contains('.')
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_')
            });
        if !valid {
            return Err(AndroidPackageError::InvalidPackageName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidPackageInfo {
    pub package_name: AndroidPackageName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidPackageError {
    InvalidPackageName,
    InvalidApkRelativePath,
}
