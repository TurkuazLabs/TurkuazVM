// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qmp_tcp_transport_tool.rs
// # 📌 Amac: QMP icin cross-platform TCP transport adapteri saglar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Windows ve Linux hostlarda CRLF sonlu QMP mesajlarini socket uzerinden tasir
// # Bagimli Oldugu Katman: Tool

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::ports::qmp_transport_port::{QmpTransportError, QmpTransportPort};

pub struct QmpTcpTransportSettings {
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}

pub struct QmpTcpTransportTool {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl QmpTcpTransportTool {
    pub fn connect(
        endpoint: SocketAddr,
        settings: &QmpTcpTransportSettings,
    ) -> Result<Self, QmpTransportError> {
        let writer = TcpStream::connect_timeout(&endpoint, settings.connect_timeout)
            .map_err(|error| QmpTransportError::ConnectFailed(error.to_string()))?;

        writer
            .set_read_timeout(Some(settings.read_timeout))
            .map_err(|error| QmpTransportError::ReadFailed(error.to_string()))?;
        writer
            .set_write_timeout(Some(settings.write_timeout))
            .map_err(|error| QmpTransportError::WriteFailed(error.to_string()))?;

        let reader_stream = writer
            .try_clone()
            .map_err(|error| QmpTransportError::ReadFailed(error.to_string()))?;

        Ok(Self {
            reader: BufReader::new(reader_stream),
            writer,
        })
    }
}

impl QmpTransportPort for QmpTcpTransportTool {
    fn read_message(&mut self) -> Result<String, QmpTransportError> {
        let mut line = String::new();
        let bytes = self
            .reader
            .read_line(&mut line)
            .map_err(|error| QmpTransportError::ReadFailed(error.to_string()))?;

        if bytes == 0 {
            return Err(QmpTransportError::ConnectionClosed);
        }

        Ok(line
            .trim_end_matches(|character| character == '\r' || character == '\n')
            .to_owned())
    }

    fn write_message(&mut self, message: &str) -> Result<(), QmpTransportError> {
        self.writer
            .write_all(message.as_bytes())
            .and_then(|()| self.writer.write_all(b"\r\n"))
            .and_then(|()| self.writer.flush())
            .map_err(|error| QmpTransportError::WriteFailed(error.to_string()))
    }
}
