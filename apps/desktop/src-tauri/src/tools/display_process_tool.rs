// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/display_process_tool.rs
// # 📌 Amac: Desktop tarafindan local TurkuazDisplay native process'ini baslatir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Display endpoint, local Engine endpoint ve opsiyonel gaming input session bilgisini ayri native process'e aktarir
// # Bagimli Oldugu Katman: Service | Tool | View

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const ENV_ENGINE_TOKEN: &str = "TURKUAZVM_DISPLAY_ENGINE_TOKEN";

pub struct DisplayProcessTool {
    executable: PathBuf,
    working_directory: PathBuf,
}

impl DisplayProcessTool {
    pub fn new(executable: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            executable: platform_executable(executable),
            working_directory,
        }
    }

    pub fn spawn(
        &self,
        vm_id: &str,
        endpoint: &str,
        title: &str,
        engine_endpoint: SocketAddr,
        engine_token: Option<&str>,
        gaming_input: bool,
    ) -> Result<(), String> {
        if !self.executable.is_file() {
            return Err(format!(
                "TurkuazDisplay binary not found: {}",
                self.executable.display()
            ));
        }
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.working_directory)
            .arg("--vm-id")
            .arg(vm_id)
            .arg("--endpoint")
            .arg(endpoint)
            .arg("--title")
            .arg(title)
            .arg("--engine-endpoint")
            .arg(engine_endpoint.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if gaming_input {
            command.arg("--gaming-input");
        }
        if let Some(token) = engine_token {
            command.env(ENV_ENGINE_TOKEN, token);
        }
        command.spawn().map(|_| ()).map_err(|error| error.to_string())
    }
}

fn platform_executable(path: PathBuf) -> PathBuf {
    if cfg!(windows) && path.extension().is_none() {
        Path::new(&format!("{}.exe", path.display())).to_path_buf()
    } else {
        path
    }
}
