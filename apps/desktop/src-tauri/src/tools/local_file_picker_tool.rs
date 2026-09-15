// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/local_file_picker_tool.rs
// # 📌 Amac: Yerel hosttan installer ISO secimini native dosya secici ile saglar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Windows OpenFileDialog ile ISO secimini Tool katmaninda izole eder
// # Bagimli Oldugu Katman: Service | Tool

#[cfg(windows)]
use std::process::Command;

pub struct LocalFilePickerTool;

impl LocalFilePickerTool {
    #[cfg(windows)]
    pub fn pick_iso() -> Result<Option<String>, String> {
        let script = r#"Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Filter='ISO Images (*.iso)|*.iso'; $d.Multiselect=$false; if($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){[Console]::Write($d.FileName)}"#;
        let output = Command::new("powershell.exe")
            .args(["-NoLogo", "-NoProfile", "-STA", "-Command", script])
            .output()
            .map_err(|error| format!("ISO file picker launch failed: {error}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if value.is_empty() { Ok(None) } else { Ok(Some(value)) }
    }

    #[cfg(not(windows))]
    pub fn pick_iso() -> Result<Option<String>, String> {
        Err(String::from("Native ISO picker is currently available on Windows hosts"))
    }
}
