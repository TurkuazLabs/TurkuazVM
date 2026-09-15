// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/config/display_config.rs
// # 📌 Amac: TurkuazDisplay process baslangic argumanlarini typed konfigurasyona donusturur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM/RFB endpoint, local Engine endpoint, gaming input modu, pencere basligi ve fullscreen tercihini yukler
// # Bagimli Oldugu Katman: Service | Tool | View

use std::net::SocketAddr;

const ARG_VM_ID: &str = "--vm-id";
const ARG_ENDPOINT: &str = "--endpoint";
const ARG_ENGINE_ENDPOINT: &str = "--engine-endpoint";
const ARG_GAMING_INPUT: &str = "--gaming-input";
const ARG_TITLE: &str = "--title";
const ARG_FULLSCREEN: &str = "--fullscreen";
const ENV_ENGINE_TOKEN: &str = "TURKUAZVM_DISPLAY_ENGINE_TOKEN";
const DEFAULT_TITLE: &str = "TurkuazVM";

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub vm_id: String,
    pub endpoint: SocketAddr,
    pub engine_endpoint: SocketAddr,
    pub engine_token: Option<String>,
    pub gaming_input: bool,
    pub title: String,
    pub fullscreen: bool,
}

impl DisplayConfig {
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let vm_id = required_value(&args, ARG_VM_ID)?;
        let endpoint = parse_loopback_endpoint(&required_value(&args, ARG_ENDPOINT)?, "display")?;
        let engine_endpoint = parse_loopback_endpoint(
            &required_value(&args, ARG_ENGINE_ENDPOINT)?,
            "Engine API",
        )?;
        let title = optional_value(&args, ARG_TITLE).unwrap_or_else(|| DEFAULT_TITLE.to_owned());
        let fullscreen = args.iter().any(|value| value == ARG_FULLSCREEN);
        let gaming_input = args.iter().any(|value| value == ARG_GAMING_INPUT);
        let engine_token = std::env::var(ENV_ENGINE_TOKEN)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        Ok(Self {
            vm_id,
            endpoint,
            engine_endpoint,
            engine_token,
            gaming_input,
            title,
            fullscreen,
        })
    }
}

fn parse_loopback_endpoint(value: &str, label: &str) -> Result<SocketAddr, String> {
    let endpoint = value
        .parse::<SocketAddr>()
        .map_err(|error| format!("Invalid {label} endpoint: {error}"))?;
    if !endpoint.ip().is_loopback() {
        return Err(format!("{label} endpoint must use loopback"));
    }
    Ok(endpoint)
}

fn required_value(args: &[String], key: &str) -> Result<String, String> {
    optional_value(args, key).ok_or_else(|| format!("Missing required argument: {key}"))
}

fn optional_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|value| value == key)
        .and_then(|index| args.get(index + 1))
        .cloned()
}
