// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/local_file_viewer_tool.rs
// # 📌 Amac: Local TurkuazVM data klasorundeki diagnostic metin dosyalarini host editorunde acar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Canonical data-root siniri ile Launcher, Engine ve Android diagnostic log goruntulemesini guvenli saglar
// # Bagimli Oldugu Katman: Service | Tool

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DATA_DIRECTORY: &str = "data";
#[cfg(windows)]
const WINDOWS_TEXT_VIEWER: &str = "notepad.exe";
#[cfg(not(windows))]
const LINUX_TEXT_VIEWER: &str = "xdg-open";

pub struct LocalFileViewerTool;

impl LocalFileViewerTool {
    pub fn open_install_log(project_root: &Path, requested_path: &str) -> Result<(), String> {
        Self::open_data_log(project_root, requested_path)
    }

    pub fn open_data_log(project_root: &Path, requested_path: &str) -> Result<(), String> {
        let data_root = canonical_existing_directory(&project_root.join(DATA_DIRECTORY))?;
        let requested_input = PathBuf::from(requested_path);
        let requested_input = if requested_input.is_absolute() {
            requested_input
        } else if requested_input.starts_with(DATA_DIRECTORY) {
            project_root.join(requested_input)
        } else {
            project_root.join(DATA_DIRECTORY).join(requested_input)
        };
        let requested = fs::canonicalize(requested_input)
            .map_err(|error| format!("install log path could not be resolved: {error}"))?;
        if !requested.starts_with(&data_root) {
            return Err(String::from("install log path is outside TurkuazVM data root"));
        }
        if !requested.is_file() {
            return Err(String::from("log file does not exist"));
        }
        let extension = requested.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
        if !matches!(extension.as_str(), "log" | "txt") {
            return Err(String::from("only .log and .txt diagnostic files can be opened"));
        }
        spawn_text_viewer(&requested)
    }
}

fn canonical_existing_directory(path: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(path).map_err(|error| format!("data root create failed: {error}"))?;
    fs::canonicalize(path).map_err(|error| format!("data root resolve failed: {error}"))
}

#[cfg(windows)]
fn spawn_text_viewer(path: &Path) -> Result<(), String> {
    Command::new(WINDOWS_TEXT_VIEWER)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Windows log viewer launch failed: {error}"))
}

#[cfg(not(windows))]
fn spawn_text_viewer(path: &Path) -> Result<(), String> {
    Command::new(LINUX_TEXT_VIEWER)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("log viewer launch failed: {error}"))
}
