// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/tools/engine_api_server_tool.rs
// # 📌 Amac: Local veya remote Engine API icin newline-delimited JSON TCP/TLS transportunu uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Request parse, cached TLS context ve response write I/O detaylarini Controller/Service katmanlarindan ayirir
// # Bagimli Oldugu Katman: Controller | Tool

use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use turkuazvm_engine_api::{EngineRequest, EngineResponse};
use turkuazvm_transport::tools::tls_stream_tool::{
    BoxedIoStream, TlsServerContext, TlsServerSettings, TlsStreamTool,
};

pub struct EngineApiConnection {
    stream: BoxedIoStream,
    pub request: EngineRequest,
}

pub struct EngineApiServerTool {
    listener: TcpListener,
    timeout: Duration,
    tls: Option<TlsServerContext>,
}

impl EngineApiServerTool {
    pub fn bind(
        ip: IpAddr,
        port: u16,
        timeout: Duration,
        tls: Option<TlsServerSettings>,
    ) -> Result<Self, String> {
        let listener = TcpListener::bind(SocketAddr::new(ip, port))
            .map_err(|error| error.to_string())?;
        listener.set_nonblocking(true).map_err(|error| error.to_string())?;
        let tls = tls
            .as_ref()
            .map(TlsStreamTool::load_server_context)
            .transpose()
            .map_err(|error| format!("TLS server config failed: {error:?}"))?;
        Ok(Self {
            listener,
            timeout,
            tls,
        })
    }

    pub fn try_accept(&self) -> Result<Option<EngineApiConnection>, String> {
        let (stream, _) = match self.listener.accept() {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        stream.set_nonblocking(false).map_err(|error| error.to_string())?;
        configure_socket(&stream, self.timeout)?;
        let mut stream: BoxedIoStream = match &self.tls {
            Some(context) => TlsStreamTool::wrap_server(stream, context)
                .map_err(|error| format!("TLS accept failed: {error:?}"))?,
            None => Box::new(stream),
        };
        let request = read_request(&mut stream)?;
        Ok(Some(EngineApiConnection { stream, request }))
    }

    pub fn respond(mut connection: EngineApiConnection, response: &EngineResponse) -> Result<(), String> {
        let mut payload = serde_json::to_vec(response).map_err(|error| error.to_string())?;
        payload.push(b'\n');
        connection
            .stream
            .write_all(&payload)
            .map_err(|error| error.to_string())?;
        connection.stream.flush().map_err(|error| error.to_string())
    }
}

fn configure_socket(stream: &TcpStream, timeout: Duration) -> Result<(), String> {
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| error.to_string())
}

fn read_request(stream: &mut BoxedIoStream) -> Result<EngineRequest, String> {
    let mut reader = BufReader::new(stream.as_mut());
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
        return Err(String::from("Engine API request is empty"));
    }
    serde_json::from_str::<EngineRequest>(line.trim_end()).map_err(|error| error.to_string())
}
