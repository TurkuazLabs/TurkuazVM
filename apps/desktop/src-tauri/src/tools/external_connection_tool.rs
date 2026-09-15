// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/external_connection_tool.rs
// # 📌 Amac: SSH/RDP baglanti hedeflerini dogrular, TCP erisilebilirligini test eder ve host istemcilerini acar
// # 📌 Modul - Rust
// # Version: 0.32.2
// # Aciklama: Connection Center icin guvenli hedef dogrulama, loopback port secimi, SSH terminali ve RDP istemcisi adaptorunu saglar
// # Bagimli Oldugu Katman: Tool

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

const LOCALHOST_NAME: &str = "localhost";
const LOOPBACK_V4_TEXT: &str = "127.0.0.1";
#[cfg(target_os = "windows")]
const SSH_EXECUTABLE_WINDOWS: &str = "powershell.exe";
#[cfg(all(unix, not(target_os = "macos")))]
const SSH_EXECUTABLE_UNIX: &str = "ssh";
#[cfg(target_os = "windows")]
const RDP_EXECUTABLE_WINDOWS: &str = "mstsc.exe";
#[cfg(all(unix, not(target_os = "macos")))]
const RDP_EXECUTABLE_LINUX: &str = "xfreerdp";
#[cfg(all(unix, not(target_os = "macos")))]
const TERMINAL_EXECUTABLE_LINUX: &str = "x-terminal-emulator";
#[cfg(target_os = "macos")]
const APPLESCRIPT_EXECUTABLE_MACOS: &str = "osascript";
const DEFAULT_PROBE_TIMEOUT_MS: u64 = 1200;
const MIN_PORT: u16 = 1;
const USERNAME_MAX_LENGTH: usize = 64;

pub struct ExternalConnectionTool;

impl ExternalConnectionTool {
    pub fn find_available_loopback_tcp_port(start_port: u16, end_port: u16) -> Result<u16, String> {
        if start_port < MIN_PORT || end_port < start_port {
            return Err(String::from("Gecersiz host port araligi"));
        }
        for port in start_port..=end_port {
            if TcpListener::bind((Ipv4Addr::LOCALHOST, port)).is_ok() {
                return Ok(port);
            }
        }
        Err(format!("Loopback TCP portu bulunamadi, aralik {start_port}-{end_port}"))
    }

    pub fn test_tcp(host: &str, port: u16) -> Result<bool, String> {
        let socket = Self::socket_addr(host, port)?;
        match TcpStream::connect_timeout(&socket, Duration::from_millis(DEFAULT_PROBE_TIMEOUT_MS)) {
            Ok(stream) => {
                drop(stream);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    pub fn open_ssh(username: &str, host: &str, port: u16) -> Result<(), String> {
        let username = Self::validate_username(username)?;
        let host = Self::normalized_host(host)?;
        Self::validate_port(port)?;

        #[cfg(target_os = "windows")]
        {
            let command = format!("ssh -p {port} {username}@{host}");
            Command::new(SSH_EXECUTABLE_WINDOWS)
                .args(["-NoExit", "-Command", &command])
                .spawn()
                .map_err(|error| format!("SSH terminali acilamadi: {error}"))?;
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let command = format!("ssh -p {port} {username}@{host}");
            let script = format!("tell application \"Terminal\" to do script \"{command}\"");
            Command::new(APPLESCRIPT_EXECUTABLE_MACOS)
                .args(["-e", &script])
                .spawn()
                .map_err(|error| format!("SSH terminali acilamadi: {error}"))?;
            return Ok(());
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let ssh_target = format!("{username}@{host}");
            Command::new(TERMINAL_EXECUTABLE_LINUX)
                .args(["-e", SSH_EXECUTABLE_UNIX, "-p", &port.to_string(), &ssh_target])
                .spawn()
                .map_err(|error| format!("SSH terminali acilamadi: {error}"))?;
            return Ok(());
        }

        #[allow(unreachable_code)]
        Err(String::from("Bu platformda SSH istemcisi acma desteklenmiyor"))
    }

    pub fn open_rdp(host: &str, port: u16) -> Result<(), String> {
        let host = Self::normalized_host(host)?;
        Self::validate_port(port)?;

        #[cfg(target_os = "windows")]
        {
            Command::new(RDP_EXECUTABLE_WINDOWS)
                .arg(format!("/v:{host}:{port}"))
                .spawn()
                .map_err(|error| format!("RDP istemcisi acilamadi: {error}"))?;
            return Ok(());
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            Command::new(RDP_EXECUTABLE_LINUX)
                .arg(format!("/v:{host}:{port}"))
                .spawn()
                .map_err(|error| format!("RDP istemcisi acilamadi: {error}"))?;
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            return Err(String::from("RDP icin macOS uzerinde kurulu bir RDP istemcisi gerekli"));
        }

        #[allow(unreachable_code)]
        Err(String::from("Bu platformda RDP istemcisi acma desteklenmiyor"))
    }

    fn socket_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
        Self::validate_port(port)?;
        let normalized = Self::normalized_host(host)?;
        let ip = if normalized == LOCALHOST_NAME {
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        } else {
            normalized.parse::<IpAddr>().map_err(|_| String::from("Baglanti hostu IP adresi veya localhost olmali"))?
        };
        Ok(SocketAddr::new(ip, port))
    }

    fn normalized_host(host: &str) -> Result<String, String> {
        let host = host.trim();
        if host.eq_ignore_ascii_case(LOCALHOST_NAME) {
            return Ok(String::from(LOCALHOST_NAME));
        }
        if host == LOOPBACK_V4_TEXT {
            return Ok(String::from(LOOPBACK_V4_TEXT));
        }
        host.parse::<IpAddr>()
            .map(|value| value.to_string())
            .map_err(|_| String::from("Baglanti hostu IP adresi veya localhost olmali"))
    }

    fn validate_username(username: &str) -> Result<String, String> {
        let username = username.trim();
        if username.is_empty() || username.len() > USERNAME_MAX_LENGTH {
            return Err(String::from("SSH kullanici adi gerekli"));
        }
        if !username.chars().all(|value| value.is_ascii_alphanumeric() || matches!(value, '.' | '_' | '-')) {
            return Err(String::from("SSH kullanici adinda desteklenmeyen karakter var"));
        }
        Ok(username.to_owned())
    }

    fn validate_port(port: u16) -> Result<(), String> {
        if port < MIN_PORT {
            return Err(String::from("Baglanti portu sifirdan buyuk olmali"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ExternalConnectionTool;

    #[test]
    fn rejects_shell_like_username() {
        let result = ExternalConnectionTool::open_ssh("root;calc", "127.0.0.1", 22);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_ip_remote_host() {
        let result = ExternalConnectionTool::test_tcp("example.invalid", 22);
        assert!(result.is_err());
    }

    #[test]
    fn finds_available_loopback_port() {
        let port = ExternalConnectionTool::find_available_loopback_tcp_port(42222, 42240)
            .expect("expected an available loopback port");
        assert!((42222..=42240).contains(&port));
    }
}
