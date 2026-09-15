// # 📄 Dosya Yolu: /turkuazvm/crates/guest-agent-protocol/src/lib.rs
// # 📌 Amac: Turkuaz Android Guest Agent TVGB v2 authenticated wire protocol modellerini ve codec'ini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Authenticated handshake, direction-key HMAC, 96-byte frame header, replay window ve JSON action payloadlarini tasir
// # Bagimli Oldugu Katman: Tool

use std::fmt;
use std::io::{Read, Write};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const GUEST_AGENT_PROTOCOL_VERSION: u16 = 2;
pub const TVGB_ROOT_KEY_LEN: usize = 32;
pub const TVGB_HEADER_LEN: usize = 96;
pub const TVGB_TAG_LEN: usize = 32;
pub const TVGB_HANDSHAKE_RECORD_LEN: usize = 72;
pub const TVGB_MAX_PAYLOAD_LEN: usize = 1024 * 1024;

const HANDSHAKE_PREFIX_LEN: usize = 40;
const CLIENT_HELLO_MAGIC: &[u8; 4] = b"TVH2";
const SERVER_HELLO_MAGIC: &[u8; 4] = b"TVS2";
const FRAME_MAGIC: &[u8; 4] = b"TVGB";
const FRAME_FLAGS_NONE: u16 = 0;
const MESSAGE_TYPE_REQUEST: u8 = 1;
const MESSAGE_TYPE_RESPONSE: u8 = 2;
const LABEL_CLIENT_HELLO: &[u8] = b"TVGBv2:client-hello";
const LABEL_H2G: &[u8] = b"TVGBv2:H2G";
const LABEL_G2H: &[u8] = b"TVGBv2:G2H";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestAgentRequest {
    pub request_id: u64,
    pub protocol_version: u16,
    #[serde(flatten)]
    pub action: GuestAgentAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GuestAgentAction {
    Ping,
    Capabilities,
    ApplyTouchFrame { contacts: Vec<GuestTouchContact> },
    ResetInput,
    ClipboardGet,
    ClipboardSet { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestTouchContact {
    pub pointer_id: u8,
    pub phase: GuestTouchPhase,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestTouchPhase {
    Down,
    Move,
    Up,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestAgentResponse {
    pub request_id: u64,
    pub protocol_version: u16,
    pub ok: bool,
    pub data: Option<GuestAgentResponseData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GuestAgentResponseData {
    Pong,
    Capabilities(GuestAgentCapabilities),
    Ack,
    ClipboardText { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestAgentCapabilities {
    pub persistent_multi_touch: bool,
    pub max_contacts: u8,
    pub continuation_api: bool,
    pub input_backend: String,
    pub secure_transport_v2: bool,
    pub clipboard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TvgbDirection {
    HostToGuest = 1,
    GuestToHost = 2,
}

impl TryFrom<u8> for TvgbDirection {
    type Error = TvgbProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::HostToGuest),
            2 => Ok(Self::GuestToHost),
            _ => Err(TvgbProtocolError::InvalidFrame("unknown direction")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvgbMessageType {
    Request,
    Response,
}

impl TvgbMessageType {
    const fn wire_value(self) -> u8 {
        match self {
            Self::Request => MESSAGE_TYPE_REQUEST,
            Self::Response => MESSAGE_TYPE_RESPONSE,
        }
    }
}

impl TryFrom<u8> for TvgbMessageType {
    type Error = TvgbProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            MESSAGE_TYPE_REQUEST => Ok(Self::Request),
            MESSAGE_TYPE_RESPONSE => Ok(Self::Response),
            _ => Err(TvgbProtocolError::InvalidFrame("unknown message type")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TvgbFrame {
    pub direction: TvgbDirection,
    pub message_type: TvgbMessageType,
    pub sequence: u64,
    pub request_id: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TvgbProtocolError {
    Io(String),
    Entropy(String),
    AuthenticationFailed,
    InvalidHandshake(&'static str),
    InvalidFrame(&'static str),
    PayloadTooLarge(usize),
    ReplayRejected(u64),
    SessionMismatch,
    DirectionMismatch,
}

impl fmt::Display for TvgbProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "TVGB I/O failed: {message}"),
            Self::Entropy(message) => write!(formatter, "TVGB entropy failed: {message}"),
            Self::AuthenticationFailed => write!(formatter, "TVGB authentication failed"),
            Self::InvalidHandshake(message) => write!(formatter, "TVGB handshake invalid: {message}"),
            Self::InvalidFrame(message) => write!(formatter, "TVGB frame invalid: {message}"),
            Self::PayloadTooLarge(size) => write!(formatter, "TVGB payload too large: {size}"),
            Self::ReplayRejected(sequence) => write!(formatter, "TVGB replay rejected: {sequence}"),
            Self::SessionMismatch => write!(formatter, "TVGB session mismatch"),
            Self::DirectionMismatch => write!(formatter, "TVGB direction mismatch"),
        }
    }
}

impl std::error::Error for TvgbProtocolError {}

pub struct TvgbSessionKeys {
    session_id: [u8; 16],
    transcript_hash: [u8; 32],
    host_to_guest: [u8; 32],
    guest_to_host: [u8; 32],
}

impl fmt::Debug for TvgbSessionKeys {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TvgbSessionKeys")
            .field("session_id", &"[redacted]")
            .field("transcript_hash", &"[redacted]")
            .field("host_to_guest", &"[redacted]")
            .field("guest_to_host", &"[redacted]")
            .finish()
    }
}

impl TvgbSessionKeys {
    #[must_use]
    pub const fn session_id(&self) -> &[u8; 16] {
        &self.session_id
    }

    #[must_use]
    pub const fn transcript_hash(&self) -> &[u8; 32] {
        &self.transcript_hash
    }

    fn key_for(&self, direction: TvgbDirection) -> &[u8; 32] {
        match direction {
            TvgbDirection::HostToGuest => &self.host_to_guest,
            TvgbDirection::GuestToHost => &self.guest_to_host,
        }
    }
}

impl Drop for TvgbSessionKeys {
    fn drop(&mut self) {
        self.session_id.fill(0);
        self.transcript_hash.fill(0);
        self.host_to_guest.fill(0);
        self.guest_to_host.fill(0);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReplayWindow {
    highest: u64,
    bitmap: u64,
    initialized: bool,
}

impl ReplayWindow {
    pub fn accept(&mut self, sequence: u64) -> bool {
        if sequence == 0 {
            return false;
        }
        if !self.initialized {
            self.highest = sequence;
            self.bitmap = 1;
            self.initialized = true;
            return true;
        }
        if sequence > self.highest {
            let shift = sequence - self.highest;
            self.bitmap = if shift >= 64 { 1 } else { (self.bitmap << shift) | 1 };
            self.highest = sequence;
            return true;
        }
        let delta = self.highest - sequence;
        if delta >= 64 {
            return false;
        }
        let mask = 1_u64 << delta;
        if self.bitmap & mask != 0 {
            return false;
        }
        self.bitmap |= mask;
        true
    }
}

pub fn client_handshake<S>(stream: &mut S, root_key: &[u8; TVGB_ROOT_KEY_LEN]) -> Result<TvgbSessionKeys, TvgbProtocolError>
where
    S: Read + Write,
{
    let mut host_nonce = [0_u8; 32];
    getrandom::getrandom(&mut host_nonce).map_err(|error| TvgbProtocolError::Entropy(error.to_string()))?;
    let client = build_client_hello(root_key, &host_nonce)?;
    stream.write_all(&client).map_err(io_error)?;
    stream.flush().map_err(io_error)?;

    let mut server = [0_u8; TVGB_HANDSHAKE_RECORD_LEN];
    stream.read_exact(&mut server).map_err(io_error)?;
    validate_server_hello(root_key, &client, &server)?;
    derive_session_keys(root_key, &client, &server)
}

pub fn write_frame<W>(
    writer: &mut W,
    keys: &TvgbSessionKeys,
    direction: TvgbDirection,
    message_type: TvgbMessageType,
    sequence: u64,
    request_id: u64,
    payload: &[u8],
) -> Result<(), TvgbProtocolError>
where
    W: Write,
{
    if sequence == 0 {
        return Err(TvgbProtocolError::InvalidFrame("sequence must be non-zero"));
    }
    if payload.len() > TVGB_MAX_PAYLOAD_LEN {
        return Err(TvgbProtocolError::PayloadTooLarge(payload.len()));
    }
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| TvgbProtocolError::PayloadTooLarge(payload.len()))?;
    let mut header = [0_u8; TVGB_HEADER_LEN];
    header[0..4].copy_from_slice(FRAME_MAGIC);
    header[4..6].copy_from_slice(&GUEST_AGENT_PROTOCOL_VERSION.to_be_bytes());
    header[6..8].copy_from_slice(&(TVGB_HEADER_LEN as u16).to_be_bytes());
    header[8] = direction as u8;
    header[9] = message_type.wire_value();
    header[10..12].copy_from_slice(&FRAME_FLAGS_NONE.to_be_bytes());
    header[12..28].copy_from_slice(keys.session_id());
    header[28..36].copy_from_slice(&sequence.to_be_bytes());
    header[36..44].copy_from_slice(&request_id.to_be_bytes());
    header[44..48].copy_from_slice(&payload_len.to_be_bytes());
    header[48..80].copy_from_slice(keys.transcript_hash());

    let tag = authenticate(keys.key_for(direction), &[&header, payload])?;
    writer.write_all(&header).map_err(io_error)?;
    writer.write_all(payload).map_err(io_error)?;
    writer.write_all(&tag).map_err(io_error)?;
    writer.flush().map_err(io_error)
}

pub fn read_frame<R>(
    reader: &mut R,
    keys: &TvgbSessionKeys,
    expected_direction: TvgbDirection,
    replay_window: &mut ReplayWindow,
) -> Result<TvgbFrame, TvgbProtocolError>
where
    R: Read,
{
    let mut header = [0_u8; TVGB_HEADER_LEN];
    reader.read_exact(&mut header).map_err(io_error)?;
    if &header[0..4] != FRAME_MAGIC {
        return Err(TvgbProtocolError::InvalidFrame("magic mismatch"));
    }
    if u16::from_be_bytes([header[4], header[5]]) != GUEST_AGENT_PROTOCOL_VERSION {
        return Err(TvgbProtocolError::InvalidFrame("version mismatch"));
    }
    if usize::from(u16::from_be_bytes([header[6], header[7]])) != TVGB_HEADER_LEN {
        return Err(TvgbProtocolError::InvalidFrame("header length mismatch"));
    }
    let direction = TvgbDirection::try_from(header[8])?;
    if direction != expected_direction {
        return Err(TvgbProtocolError::DirectionMismatch);
    }
    let message_type = TvgbMessageType::try_from(header[9])?;
    if &header[12..28] != keys.session_id() || &header[48..80] != keys.transcript_hash() {
        return Err(TvgbProtocolError::SessionMismatch);
    }
    let sequence = u64::from_be_bytes(header[28..36].try_into().expect("fixed sequence slice"));
    let request_id = u64::from_be_bytes(header[36..44].try_into().expect("fixed request slice"));
    let payload_len = u32::from_be_bytes(header[44..48].try_into().expect("fixed length slice")) as usize;
    if payload_len > TVGB_MAX_PAYLOAD_LEN {
        return Err(TvgbProtocolError::PayloadTooLarge(payload_len));
    }
    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload).map_err(io_error)?;
    let mut tag = [0_u8; TVGB_TAG_LEN];
    reader.read_exact(&mut tag).map_err(io_error)?;

    verify_authentication(keys.key_for(direction), &[&header, &payload], &tag)?;
    if !replay_window.accept(sequence) {
        return Err(TvgbProtocolError::ReplayRejected(sequence));
    }

    Ok(TvgbFrame { direction, message_type, sequence, request_id, payload })
}

fn build_client_hello(root_key: &[u8; TVGB_ROOT_KEY_LEN], nonce: &[u8; 32]) -> Result<[u8; TVGB_HANDSHAKE_RECORD_LEN], TvgbProtocolError> {
    let mut record = [0_u8; TVGB_HANDSHAKE_RECORD_LEN];
    record[0..4].copy_from_slice(CLIENT_HELLO_MAGIC);
    record[4..6].copy_from_slice(&GUEST_AGENT_PROTOCOL_VERSION.to_be_bytes());
    record[8..40].copy_from_slice(nonce);
    let tag = authenticate(root_key, &[LABEL_CLIENT_HELLO, &record[..HANDSHAKE_PREFIX_LEN]])?;
    record[HANDSHAKE_PREFIX_LEN..].copy_from_slice(&tag);
    Ok(record)
}

fn validate_server_hello(
    root_key: &[u8; TVGB_ROOT_KEY_LEN],
    client: &[u8; TVGB_HANDSHAKE_RECORD_LEN],
    server: &[u8; TVGB_HANDSHAKE_RECORD_LEN],
) -> Result<(), TvgbProtocolError> {
    if &server[0..4] != SERVER_HELLO_MAGIC {
        return Err(TvgbProtocolError::InvalidHandshake("server magic mismatch"));
    }
    if u16::from_be_bytes([server[4], server[5]]) != GUEST_AGENT_PROTOCOL_VERSION {
        return Err(TvgbProtocolError::InvalidHandshake("server version mismatch"));
    }
    verify_authentication(
        root_key,
        &[client, &server[..HANDSHAKE_PREFIX_LEN]],
        &server[HANDSHAKE_PREFIX_LEN..],
    )
}

fn derive_session_keys(
    root_key: &[u8; TVGB_ROOT_KEY_LEN],
    client: &[u8; TVGB_HANDSHAKE_RECORD_LEN],
    server: &[u8; TVGB_HANDSHAKE_RECORD_LEN],
) -> Result<TvgbSessionKeys, TvgbProtocolError> {
    let mut digest = Sha256::new();
    digest.update(client);
    digest.update(server);
    let transcript: [u8; 32] = digest.finalize().into();
    let host_to_guest = authenticate(root_key, &[LABEL_H2G, &transcript])?;
    let guest_to_host = authenticate(root_key, &[LABEL_G2H, &transcript])?;
    let mut session_id = [0_u8; 16];
    session_id.copy_from_slice(&transcript[..16]);
    Ok(TvgbSessionKeys { session_id, transcript_hash: transcript, host_to_guest, guest_to_host })
}

fn authenticate(key: &[u8], parts: &[&[u8]]) -> Result<[u8; TVGB_TAG_LEN], TvgbProtocolError> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| TvgbProtocolError::AuthenticationFailed)?;
    for part in parts {
        mac.update(part);
    }
    let bytes = mac.finalize().into_bytes();
    let mut output = [0_u8; TVGB_TAG_LEN];
    output.copy_from_slice(&bytes);
    Ok(output)
}

fn verify_authentication(key: &[u8], parts: &[&[u8]], tag: &[u8]) -> Result<(), TvgbProtocolError> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| TvgbProtocolError::AuthenticationFailed)?;
    for part in parts {
        mac.update(part);
    }
    mac.verify_slice(tag).map_err(|_| TvgbProtocolError::AuthenticationFailed)
}

fn io_error(error: std::io::Error) -> TvgbProtocolError {
    TvgbProtocolError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn request_round_trip_preserves_protocol_version() {
        let request = GuestAgentRequest {
            request_id: 42,
            protocol_version: GUEST_AGENT_PROTOCOL_VERSION,
            action: GuestAgentAction::ApplyTouchFrame {
                contacts: vec![GuestTouchContact { pointer_id: 2, phase: GuestTouchPhase::Move, x: 8_000, y: 5_000 }],
            },
        };
        let encoded = serde_json::to_string(&request).expect("guest request serialization");
        let decoded: GuestAgentRequest = serde_json::from_str(&encoded).expect("guest request parse");
        assert_eq!(decoded, request);
    }

    #[test]
    fn replay_window_accepts_out_of_order_once_and_rejects_duplicate() {
        let mut window = ReplayWindow::default();
        assert!(window.accept(10));
        assert!(window.accept(12));
        assert!(window.accept(11));
        assert!(!window.accept(11));
        assert!(!window.accept(0));
    }

    #[test]
    fn authenticated_frame_rejects_tamper_before_replay_state_moves() {
        let keys = TvgbSessionKeys {
            session_id: [1; 16],
            transcript_hash: [2; 32],
            host_to_guest: [3; 32],
            guest_to_host: [4; 32],
        };
        let mut wire = Vec::new();
        write_frame(
            &mut wire,
            &keys,
            TvgbDirection::HostToGuest,
            TvgbMessageType::Request,
            7,
            99,
            br#"{"action":"ping"}"#,
        )
        .expect("frame write");
        let payload_offset = TVGB_HEADER_LEN;
        wire[payload_offset] ^= 0x01;
        let mut replay = ReplayWindow::default();
        let error = read_frame(&mut Cursor::new(&wire), &keys, TvgbDirection::HostToGuest, &mut replay)
            .expect_err("tampered frame must fail");
        assert_eq!(error, TvgbProtocolError::AuthenticationFailed);
        assert!(replay.accept(7));
    }
}
