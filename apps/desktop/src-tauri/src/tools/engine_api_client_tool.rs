// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/engine_api_client_tool.rs
// # 📌 Amac: Desktop Service icin local veya remote Engine API TCP/TLS client adapterini uygular
// # 📌 Modul - Rust
// # Version: 0.39.5
// # Aciklama: Connect/write/read asamalarini ayirarak mutating request retry riskini kapatir; JSON-line I/O, TLS ve auth detaylarini UI'dan gizler
// # Bagimli Oldugu Katman: Service | Tool

use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use turkuazvm_engine_api::{EngineRequest, EngineResponse};
use turkuazvm_transport::tools::tls_stream_tool::{
    BoxedIoStream, TlsClientContext, TlsClientSettings, TlsStreamTool,
};

#[derive(Debug)]
pub enum EngineApiClientError {
    Connect(String),
    Write(String),
    Read(String),
    Protocol(String),
    Tls(String),
}

impl fmt::Display for EngineApiClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(message) => write!(formatter, "connect failed: {message}"),
            Self::Write(message) => write!(formatter, "request write failed: {message}"),
            Self::Read(message) => write!(formatter, "response read failed: {message}"),
            Self::Protocol(message) => write!(formatter, "protocol failed: {message}"),
            Self::Tls(message) => write!(formatter, "TLS failed: {message}"),
        }
    }
}

impl std::error::Error for EngineApiClientError {}

#[derive(Clone)]
pub struct EngineApiClientTool {
    endpoint: SocketAddr,
    auth_token: Option<String>,
    tls: Option<TlsClientContext>,
}

impl EngineApiClientTool {
    pub fn new(
        endpoint: SocketAddr,
        auth_token: Option<String>,
        tls: Option<TlsClientSettings>,
    ) -> Result<Self, EngineApiClientError> {
        let tls = tls
            .as_ref()
            .map(TlsStreamTool::load_client_context)
            .transpose()
            .map_err(|error| EngineApiClientError::Tls(format!("{error:?}")))?;
        Ok(Self {
            endpoint,
            auth_token,
            tls,
        })
    }

    pub fn send_with_timeout(
        &self,
        request: &EngineRequest,
        timeout: Duration,
    ) -> Result<EngineResponse, EngineApiClientError> {
        let stream = TcpStream::connect_timeout(&self.endpoint, timeout)
            .map_err(|error| EngineApiClientError::Connect(error.to_string()))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|error| EngineApiClientError::Read(error.to_string()))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|error| EngineApiClientError::Write(error.to_string()))?;
        let mut stream: BoxedIoStream = match &self.tls {
            Some(context) => TlsStreamTool::wrap_client(stream, context)
                .map_err(|error| EngineApiClientError::Tls(format!("{error:?}")))?,
            None => Box::new(stream),
        };
        let mut authenticated_request = request.clone();
        authenticated_request.auth_token.clone_from(&self.auth_token);

        let mut payload = serde_json::to_vec(&authenticated_request)
            .map_err(|error| EngineApiClientError::Protocol(error.to_string()))?;
        payload.push(b'\n');
        stream
            .write_all(&payload)
            .map_err(|error| EngineApiClientError::Write(error.to_string()))?;
        stream
            .flush()
            .map_err(|error| EngineApiClientError::Write(error.to_string()))?;

        let mut reader = BufReader::new(stream.as_mut());
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| EngineApiClientError::Read(error.to_string()))?;
        if line.trim().is_empty() {
            return Err(EngineApiClientError::Protocol(String::from(
                "Engine returned an empty response",
            )));
        }
        serde_json::from_str(line.trim_end())
            .map_err(|error| EngineApiClientError::Protocol(error.to_string()))
    }
}
