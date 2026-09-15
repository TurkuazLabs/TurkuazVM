// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_android_profile_repository.rs
// # 📌 Amac: Android runtime profile bilgisini VM klasorundeki android.yml dosyasinda saklar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Android bounded context persistence portunu serde/YAML adapteri ile uygular
// # Bagimli Oldugu Katman: Repo

use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use turkuazvm_android::domain::runtime_profile::{
    AndroidDisplayProfile, AndroidRuntimeProfile, AndroidVmId,
};
use turkuazvm_android::ports::android_profile_repository_port::{
    AndroidProfileRepositoryError, AndroidProfileRepositoryPort,
};

const PROFILE_SCHEMA_VERSION: u16 = 1;
const DIR_MACHINES: &str = "machines";
const FILE_ANDROID_PROFILE: &str = "android.yml";

#[derive(Debug, Clone)]
pub struct YamlAndroidProfileRepository {
    data_root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct AndroidProfileManifest {
    schema_version: u16,
    vm_id: String,
    adb: AdbManifest,
    display: DisplayManifest,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdbManifest {
    host_ip: String,
    host_port: u16,
    guest_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct DisplayManifest {
    width: u32,
    height: u32,
    density_dpi: u32,
    target_fps: u16,
}

impl YamlAndroidProfileRepository {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }

    fn machine_dir(&self, vm_id: &AndroidVmId) -> PathBuf {
        self.data_root.join(DIR_MACHINES).join(vm_id.as_str())
    }

    fn profile_path(&self, vm_id: &AndroidVmId) -> PathBuf {
        self.machine_dir(vm_id).join(FILE_ANDROID_PROFILE)
    }

    fn manifest_header(path: &Path) -> String {
        format!(
            "# \u{1F4C4} Dosya Yolu: {}\n# \u{1F4CC} Amac: Bu Android VM icin runtime ve ADB profile bilgisini saklar\n# \u{1F4CC} Modul - YAML\n# Version: {}\n# Aciklama: ADB loopback endpoint ve Android display profilini kalici olarak tutar\n# Bagimli Oldugu Katman: Repo\n\n",
            path.display(),
            env!("CARGO_PKG_VERSION")
        )
    }

    fn read_profile(
        &self,
        path: &Path,
    ) -> Result<AndroidRuntimeProfile, AndroidProfileRepositoryError> {
        let content = fs::read_to_string(path)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        let manifest: AndroidProfileManifest = serde_yaml_ng::from_str(&content)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        if manifest.schema_version != PROFILE_SCHEMA_VERSION {
            return Err(AndroidProfileRepositoryError::Storage(format!(
                "Unsupported android profile schema: {}",
                manifest.schema_version
            )));
        }
        let host_ip = manifest
            .adb
            .host_ip
            .parse::<Ipv4Addr>()
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        let vm_id = AndroidVmId::parse(manifest.vm_id)
            .map_err(|error| AndroidProfileRepositoryError::Storage(format!("{error:?}")))?;
        let display = AndroidDisplayProfile::create(
            manifest.display.width,
            manifest.display.height,
            manifest.display.density_dpi,
            manifest.display.target_fps,
        )
        .map_err(|error| AndroidProfileRepositoryError::Storage(format!("{error:?}")))?;
        AndroidRuntimeProfile::create(
            vm_id,
            host_ip,
            manifest.adb.host_port,
            manifest.adb.guest_port,
            display,
        )
        .map_err(|error| AndroidProfileRepositoryError::Storage(format!("{error:?}")))
    }
}

impl AndroidProfileRepositoryPort for YamlAndroidProfileRepository {
    fn get(
        &self,
        vm_id: &AndroidVmId,
    ) -> Result<AndroidRuntimeProfile, AndroidProfileRepositoryError> {
        let path = self.profile_path(vm_id);
        if !path.is_file() {
            return Err(AndroidProfileRepositoryError::NotFound(vm_id.clone()));
        }
        self.read_profile(&path)
    }

    fn save(
        &mut self,
        profile: AndroidRuntimeProfile,
    ) -> Result<(), AndroidProfileRepositoryError> {
        let directory = self.machine_dir(profile.vm_id());
        fs::create_dir_all(&directory)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        let path = self.profile_path(profile.vm_id());
        let manifest = AndroidProfileManifest {
            schema_version: PROFILE_SCHEMA_VERSION,
            vm_id: profile.vm_id().as_str().to_owned(),
            adb: AdbManifest {
                host_ip: profile.adb_host_ip().to_string(),
                host_port: profile.adb_host_port(),
                guest_port: profile.adb_guest_port(),
            },
            display: DisplayManifest {
                width: profile.display().width(),
                height: profile.display().height(),
                density_dpi: profile.display().density_dpi(),
                target_fps: profile.display().target_fps(),
            },
        };
        let yaml = serde_yaml_ng::to_string(&manifest)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        fs::write(&path, format!("{}{}", Self::manifest_header(&path), yaml))
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))
    }

    fn delete(&mut self, vm_id: &AndroidVmId) -> Result<(), AndroidProfileRepositoryError> {
        let path = self.profile_path(vm_id);
        if !path.exists() {
            return Ok(());
        }
        fs::remove_file(path)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))
    }

    fn list(&self) -> Result<Vec<AndroidRuntimeProfile>, AndroidProfileRepositoryError> {
        let root = self.data_root.join(DIR_MACHINES);
        if !root.is_dir() {
            return Ok(Vec::new());
        }
        let entries = fs::read_dir(root)
            .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
        let mut profiles = Vec::new();
        for entry in entries {
            let entry = entry
                .map_err(|error| AndroidProfileRepositoryError::Storage(error.to_string()))?;
            let path = entry.path().join(FILE_ANDROID_PROFILE);
            if path.is_file() {
                profiles.push(self.read_profile(&path)?);
            }
        }
        profiles.sort_by(|left, right| left.vm_id().as_str().cmp(right.vm_id().as_str()));
        Ok(profiles)
    }
}
