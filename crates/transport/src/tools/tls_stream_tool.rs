// # 📄 Dosya Yolu: /turkuazvm/crates/transport/src/tools/tls_stream_tool.rs
// # 📌 Amac: Engine ve Desktop icin rustls tabanli TLS config ve stream islemlerini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: PEM dosyalarini bir kez context'e yukler ve her TCP baglantisini dogrulanmis TLS stream'e sarar
// # Bagimli Oldugu Katman: Tool

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned};

pub trait IoStream: Read + Write + Send {}
impl<T> IoStream for T where T: Read + Write + Send {}

pub type BoxedIoStream = Box<dyn IoStream>;

#[derive(Debug, Clone)]
pub struct TlsServerSettings {
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TlsClientSettings {
    pub ca_certificate_path: PathBuf,
    pub server_name: String,
}

#[derive(Clone)]
pub struct TlsServerContext {
    config: Arc<ServerConfig>,
}

#[derive(Clone)]
pub struct TlsClientContext {
    config: Arc<ClientConfig>,
    server_name: String,
}

#[derive(Debug)]
pub enum TlsStreamError {
    FileOpen(String),
    Pem(String),
    InvalidCertificate(String),
    MissingPrivateKey,
    InvalidServerName,
    Tls(String),
}

pub struct TlsStreamTool;

impl TlsStreamTool {
    pub fn load_server_context(
        settings: &TlsServerSettings,
    ) -> Result<TlsServerContext, TlsStreamError> {
        install_crypto_provider();
        let cert_file = File::open(&settings.certificate_path)
            .map_err(|error| TlsStreamError::FileOpen(error.to_string()))?;
        let key_file = File::open(&settings.private_key_path)
            .map_err(|error| TlsStreamError::FileOpen(error.to_string()))?;
        let mut cert_reader = BufReader::new(cert_file);
        let mut key_reader = BufReader::new(key_file);
        let certificates = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| TlsStreamError::Pem(error.to_string()))?;
        if certificates.is_empty() {
            return Err(TlsStreamError::InvalidCertificate(String::from(
                "TLS certificate file contains no certificates",
            )));
        }
        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|error| TlsStreamError::Pem(error.to_string()))?
            .ok_or(TlsStreamError::MissingPrivateKey)?;
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, private_key)
            .map_err(|error| TlsStreamError::Tls(error.to_string()))?;
        Ok(TlsServerContext {
            config: Arc::new(config),
        })
    }

    pub fn load_client_context(
        settings: &TlsClientSettings,
    ) -> Result<TlsClientContext, TlsStreamError> {
        install_crypto_provider();
        ServerName::try_from(settings.server_name.clone())
            .map_err(|_| TlsStreamError::InvalidServerName)?;
        let ca_file = File::open(&settings.ca_certificate_path)
            .map_err(|error| TlsStreamError::FileOpen(error.to_string()))?;
        let mut ca_reader = BufReader::new(ca_file);
        let mut roots = RootCertStore::empty();
        let certificates = rustls_pemfile::certs(&mut ca_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| TlsStreamError::Pem(error.to_string()))?;
        if certificates.is_empty() {
            return Err(TlsStreamError::InvalidCertificate(String::from(
                "TLS CA file contains no certificates",
            )));
        }
        for certificate in certificates {
            roots
                .add(certificate)
                .map_err(|error| TlsStreamError::InvalidCertificate(error.to_string()))?;
        }
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Ok(TlsClientContext {
            config: Arc::new(config),
            server_name: settings.server_name.clone(),
        })
    }

    pub fn wrap_server(
        stream: TcpStream,
        context: &TlsServerContext,
    ) -> Result<BoxedIoStream, TlsStreamError> {
        let connection = ServerConnection::new(Arc::clone(&context.config))
            .map_err(|error| TlsStreamError::Tls(error.to_string()))?;
        Ok(Box::new(StreamOwned::new(connection, stream)))
    }

    pub fn wrap_client(
        stream: TcpStream,
        context: &TlsClientContext,
    ) -> Result<BoxedIoStream, TlsStreamError> {
        let server_name = ServerName::try_from(context.server_name.clone())
            .map_err(|_| TlsStreamError::InvalidServerName)?;
        let connection = ClientConnection::new(Arc::clone(&context.config), server_name)
            .map_err(|error| TlsStreamError::Tls(error.to_string()))?;
        Ok(Box::new(StreamOwned::new(connection, stream)))
    }
}

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
