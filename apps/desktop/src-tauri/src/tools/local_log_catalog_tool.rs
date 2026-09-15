// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/local_log_catalog_tool.rs
// # 📌 Amac: TurkuazVM local diagnostic log dosyalarini guvenli bir katalog olarak listeler
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: data/logs ve Android image install.log dosyalarini path siniri ve metadata ile toplar
// # Bagimli Oldugu Katman: Service | Tool

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct LocalLogEntry {
    pub relative_path: String,
    pub category: String,
    pub size_bytes: u64,
    pub modified_at_unix_ms: u64,
}

pub struct LocalLogCatalogTool;

impl LocalLogCatalogTool {
    pub fn list(project_root: &Path) -> Result<Vec<LocalLogEntry>, String> {
        let data_root = project_root.join("data");
        fs::create_dir_all(&data_root).map_err(|error| format!("data root create failed: {error}"))?;
        let mut entries = Vec::new();
        Self::walk(&data_root.join("logs"), &data_root, "runtime", &mut entries)?;
        Self::walk_install_logs(&data_root.join("android-image-builds"), &data_root, &mut entries)?;
        entries.sort_by(|left, right| right.modified_at_unix_ms.cmp(&left.modified_at_unix_ms));
        Ok(entries)
    }

    fn walk(root: &Path, data_root: &Path, category: &str, output: &mut Vec<LocalLogEntry>) -> Result<(), String> {
        if !root.exists() {
            return Ok(());
        }
        for item in fs::read_dir(root).map_err(|error| format!("log directory read failed: {error}"))? {
            let item = item.map_err(|error| format!("log directory entry failed: {error}"))?;
            let path = item.path();
            if path.is_dir() {
                Self::walk(&path, data_root, category, output)?;
            } else if is_log_file(&path) {
                output.push(entry_from_path(&path, data_root, category)?);
            }
        }
        Ok(())
    }

    fn walk_install_logs(root: &Path, data_root: &Path, output: &mut Vec<LocalLogEntry>) -> Result<(), String> {
        if !root.exists() {
            return Ok(());
        }
        for item in fs::read_dir(root).map_err(|error| format!("Android log directory read failed: {error}"))? {
            let item = item.map_err(|error| format!("Android log directory entry failed: {error}"))?;
            let path = item.path();
            if path.is_dir() {
                Self::walk_install_logs(&path, data_root, output)?;
            } else if path.file_name().and_then(|value| value.to_str()) == Some("install.log") {
                output.push(entry_from_path(&path, data_root, "android_image")?);
            }
        }
        Ok(())
    }
}

fn is_log_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("log") | Some("txt"))
}

fn entry_from_path(path: &Path, data_root: &Path, category: &str) -> Result<LocalLogEntry, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("log metadata failed: {error}"))?;
    let relative = path.strip_prefix(data_root).map_err(|_| String::from("log path escaped data root"))?;
    let modified_at_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default();
    Ok(LocalLogEntry {
        relative_path: relative.to_string_lossy().replace('\\', "/"),
        category: category.to_owned(),
        size_bytes: metadata.len(),
        modified_at_unix_ms,
    })
}
