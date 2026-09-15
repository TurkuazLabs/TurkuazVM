// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/external_url_opener_tool.rs
// # 📌 Amac: Resmi installer medya sayfalarini sistem tarayicisinda guvenli sekilde acar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Yalniz HTTPS URL kabul eder ve platformun varsayilan tarayici mekanizmasini kullanir
// # Bagimli Oldugu Katman: Tool

use std::process::Command;

pub struct ExternalUrlOpenerTool;

impl ExternalUrlOpenerTool {
    pub fn open_https(url: &str) -> Result<(), String> {
        let url = url.trim();
        if !url.starts_with("https://") || url.len() > 2048 || url.chars().any(|value| matches!(value, '\r' | '\n' | '\0')) {
            return Err(String::from("Only safe HTTPS URLs are allowed"));
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer.exe")
                .arg(url)
                .spawn()
                .map_err(|error| format!("browser open failed: {error}"))?;
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(url)
                .spawn()
                .map_err(|error| format!("browser open failed: {error}"))?;
            return Ok(());
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            Command::new("xdg-open")
                .arg(url)
                .spawn()
                .map_err(|error| format!("browser open failed: {error}"))?;
            return Ok(());
        }
        #[allow(unreachable_code)]
        Err(String::from("browser open is unsupported on this platform"))
    }
}
