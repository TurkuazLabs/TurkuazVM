// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/http_download_tool.rs
// # 📌 Amac: Guest source discovery ve installer media indirmeleri icin tek curl tabanli HTTP adaptorunu saglar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Text fetch, contextual header/referer, range probe ve resumable file download curl davranisini merkezilestirir
// # Bagimli Oldugu Katman: Tool

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct HttpDownloadSettings {
    pub curl_binary: PathBuf,
    pub connect_timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone, Default)]
pub struct HttpRequestContext {
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub headers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HttpDownloadTool {
    settings: HttpDownloadSettings,
}

impl HttpDownloadTool {
    pub const fn new(settings: HttpDownloadSettings) -> Self {
        Self { settings }
    }

    pub fn fetch_text(&self, url: &str) -> Result<String, String> {
        self.fetch_text_with_context(url, &HttpRequestContext::default())
    }

    pub fn fetch_text_with_context(
        &self,
        url: &str,
        context: &HttpRequestContext,
    ) -> Result<String, String> {
        let mut command = self.base_command();
        apply_context(&mut command, context);
        let output = command
            .arg("--max-time")
            .arg(self.settings.connect_timeout_seconds.saturating_mul(4).max(20).to_string())
            .arg(url)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("curl baslatilamadi: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "HTTP fetch basarisiz: url={url} curl_exit={} detail={}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        String::from_utf8(output.stdout).map_err(|_| String::from("HTTP response UTF-8 degil"))
    }

    pub fn probe_range(
        &self,
        url: &str,
        range: &str,
        max_filesize_bytes: u64,
        context: &HttpRequestContext,
    ) -> Result<u16, String> {
        let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let mut command = self.base_command();
        apply_context(&mut command, context);
        let output = command
            .arg("--range")
            .arg(range)
            .arg("--output")
            .arg(null_device)
            .arg("--max-filesize")
            .arg(max_filesize_bytes.to_string())
            .args(["--write-out", "%{http_code}"])
            .arg(url)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|error| format!("curl probe baslatilamadi: {error}"))?;
        let code = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u16>()
            .map_err(|_| String::from("HTTP probe status code parse edilemedi"))?;
        Ok(code)
    }

    pub fn spawn_file_download(&self, url: &str, output_path: &Path) -> Result<Child, String> {
        self.spawn_file_download_with_context(
            url,
            output_path,
            true,
            &HttpRequestContext::default(),
        )
    }

    pub fn spawn_file_download_with_context(
        &self,
        url: &str,
        output_path: &Path,
        resume: bool,
        context: &HttpRequestContext,
    ) -> Result<Child, String> {
        let mut command = self.base_command();
        apply_context(&mut command, context);
        if resume {
            command.arg("--continue-at").arg("-");
        }
        command.arg("--output").arg(output_path).arg(url);
        command
            .spawn()
            .map_err(|error| format!("curl download baslatilamadi: {error}"))
    }

    pub fn build_file_download_command(
        &self,
        url: &str,
        output_path: &Path,
        resume: bool,
        context: &HttpRequestContext,
    ) -> Command {
        let mut command = self.base_command();
        apply_context(&mut command, context);
        if resume {
            command.arg("--continue-at").arg("-");
        }
        command.arg("--output").arg(output_path).arg(url);
        command
    }

    pub fn fetch_headers_with_context(
        &self,
        url: &str,
        context: &HttpRequestContext,
    ) -> Result<String, String> {
        let mut command = self.base_command();
        apply_context(&mut command, context);
        let output = command
            .arg("--head")
            .arg(url)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("curl header fetch baslatilamadi: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "HTTP header fetch basarisiz: url={url} curl_exit={}",
                output.status.code().unwrap_or(-1)
            ));
        }
        String::from_utf8(output.stdout).map_err(|_| String::from("HTTP header response UTF-8 degil"))
    }

    fn base_command(&self) -> Command {
        let mut command = Command::new(&self.settings.curl_binary);
        command
            .arg("--location")
            .arg("--fail")
            .arg("--silent")
            .arg("--show-error")
            .arg("--connect-timeout")
            .arg(self.settings.connect_timeout_seconds.to_string())
            .arg("--retry")
            .arg(self.settings.retry_count.to_string())
            .arg("--retry-delay")
            .arg(self.settings.retry_delay_seconds.to_string())
            .arg("--retry-all-errors");
        command
    }
}

fn apply_context(command: &mut Command, context: &HttpRequestContext) {
    if let Some(user_agent) = context.user_agent.as_deref() {
        command.arg("--user-agent").arg(user_agent);
    }
    if let Some(referer) = context.referer.as_deref() {
        command.arg("--referer").arg(referer);
    }
    for header in &context.headers {
        command.arg("--header").arg(header);
    }
}
