// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/domain/qmp.rs
// # 📌 Amac: QMP wire protokolu icin serde veri modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Greeting, response, event, error ve argument tasiyan QMP command request tiplerini modeller
// # Bagimli Oldugu Katman: Tool

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct QmpGreetingEnvelope {
    #[serde(rename = "QMP")]
    pub qmp: QmpGreeting,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpGreeting {
    pub version: QmpVersionInfo,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpVersionInfo {
    pub qemu: QmpSemanticVersion,
    pub package: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpSemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub micro: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpCommandInfo {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpResponseEnvelope {
    #[serde(rename = "return")]
    pub return_value: Option<Value>,
    pub error: Option<QmpErrorPayload>,
    pub id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpErrorPayload {
    pub class: String,
    pub desc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QmpEventEnvelope {
    pub event: String,
}

#[derive(Debug, Serialize)]
pub struct QmpRequest<'a> {
    pub execute: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
    pub id: u64,
}
