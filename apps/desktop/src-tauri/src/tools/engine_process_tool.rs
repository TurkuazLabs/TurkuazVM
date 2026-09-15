// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/engine_process_tool.rs
// # 📌 Amac: Desktop tarafindan local TurkuazVM Engine process bootstrap ve supervision islemini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Engine binary'yi ayni config ile baslatir, process handle'ini korur ve stdout/stderr loglarini kalici olarak kaydeder
// # Bagimli Oldugu Katman: Service | Tool

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const CONFIG_ENV: &str = "TURKUAZVM_CONFIG";
const ENGINE_LOG_RELATIVE_DIRECTORY: &str = "data/logs/engine";
const ENGINE_LOG_FILE_PREFIX: &str = "engine";
const DIAGNOSTIC_LINE_LIMIT: usize = 40;

pub struct EngineProcessHandle {
    child: Child,
    log_path: PathBuf,
}

impl EngineProcessHandle {
    pub fn process_id(&self) -> u32 {
        self.child.id()
    }

    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, String> {
        self.child.try_wait().map_err(|error| error.to_string())
    }

    pub fn diagnostic_tail(&self) -> String {
        let Ok(content) = fs::read_to_string(&self.log_path) else {
            return String::from("Engine logu okunamadi");
        };
        let lines = content.lines().collect::<Vec<_>>();
        let start = lines.len().saturating_sub(DIAGNOSTIC_LINE_LIMIT);
        lines[start..].join(" | ")
    }
}

pub struct EngineProcessTool {
    executable: PathBuf,
    config_path: PathBuf,
    working_directory: PathBuf,
}

impl EngineProcessTool {
    pub fn new(executable: PathBuf, config_path: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            executable: platform_executable(executable),
            config_path,
            working_directory,
        }
    }

    pub fn spawn(&self) -> Result<EngineProcessHandle, String> {
        if !self.executable.is_file() {
            return Err(format!(
                "Engine binary not found: {}",
                self.executable.display()
            ));
        }

        let log_directory = self.working_directory.join(ENGINE_LOG_RELATIVE_DIRECTORY);
        fs::create_dir_all(&log_directory).map_err(|error| {
            format!(
                "Engine log dizini olusturulamadi {}: {error}",
                log_directory.display()
            )
        })?;
        let log_path = log_directory.join(engine_log_file_name());
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("Engine log dosyasi acilamadi: {error}"))?;
        writeln!(
            log_file,
            "TurkuazVM Engine bootstrap\nExecutable: {}\nConfig: {}\nWorkingDirectory: {}",
            self.executable.display(),
            self.config_path.display(),
            self.working_directory.display()
        )
        .map_err(|error| format!("Engine log basligi yazilamadi: {error}"))?;
        log_file
            .flush()
            .map_err(|error| format!("Engine log basligi flush edilemedi: {error}"))?;
        let stdout_file = log_file
            .try_clone()
            .map_err(|error| format!("Engine stdout log handle kopyalanamadi: {error}"))?;

        let child = Command::new(&self.executable)
            .current_dir(&self.working_directory)
            .env(CONFIG_ENV, &self.config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(log_file))
            .spawn()
            .map_err(|error| {
                format!(
                    "Engine process baslatilamadi: {error}. Log: {}",
                    log_path.display()
                )
            })?;

        Ok(EngineProcessHandle { child, log_path })
    }
}

fn engine_log_file_name() -> String {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!(
        "{ENGINE_LOG_FILE_PREFIX}-{timestamp_ms}-desktop-{}.log",
        std::process::id()
    )
}

fn platform_executable(path: PathBuf) -> PathBuf {
    if cfg!(windows) && path.extension().is_none() {
        Path::new(&format!("{}.exe", path.display())).to_path_buf()
    } else {
        path
    }
}
