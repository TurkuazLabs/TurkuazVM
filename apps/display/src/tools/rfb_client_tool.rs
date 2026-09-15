// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/rfb_client_tool.rs
// # 📌 Amac: QEMU loopback RFB serverindan framebuffer okuyup klavye/fare inputu gonderir
// # 📌 Modul - Rust
// # Version: 0.40.2
// # Aciklama: RFB 3.3/3.7/3.8 None-auth handshake, raw framebuffer, DesktopSize, input ve atomik update/drop/byte telemetrisini native display icin uygular
// # Bagimli Oldugu Katman: Service | Tool | View

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const RFB_VERSION_3_3: &[u8; 12] = b"RFB 003.003\n";
const RFB_VERSION_3_7: &[u8; 12] = b"RFB 003.007\n";
const RFB_VERSION_3_8: &[u8; 12] = b"RFB 003.008\n";
const SECURITY_NONE: u8 = 1;
const CLIENT_SHARED: u8 = 1;
const MESSAGE_SET_PIXEL_FORMAT: u8 = 0;
const MESSAGE_SET_ENCODINGS: u8 = 2;
const MESSAGE_FRAMEBUFFER_UPDATE_REQUEST: u8 = 3;
const MESSAGE_KEY_EVENT: u8 = 4;
const MESSAGE_POINTER_EVENT: u8 = 5;
const SERVER_FRAMEBUFFER_UPDATE: u8 = 0;
const SERVER_SET_COLOR_MAP: u8 = 1;
const SERVER_BELL: u8 = 2;
const SERVER_CUT_TEXT: u8 = 3;
const ENCODING_RAW: i32 = 0;
const ENCODING_DESKTOP_SIZE: i32 = -223;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const FRAME_QUEUE_DEPTH: usize = 2;
const MAX_SERVER_NAME_BYTES: usize = 1024 * 1024;
const MAX_CUT_TEXT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RfbProtocolVersion {
    V3_3,
    V3_7,
    V3_8,
}

impl RfbProtocolVersion {
    fn negotiate(server_minor: u16) -> Self {
        match server_minor {
            7 => Self::V3_7,
            8 => Self::V3_8,
            _ => Self::V3_3,
        }
    }

    fn banner(self) -> &'static [u8; 12] {
        match self {
            Self::V3_3 => RFB_VERSION_3_3,
            Self::V3_7 => RFB_VERSION_3_7,
            Self::V3_8 => RFB_VERSION_3_8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RfbTelemetrySnapshot {
    pub framebuffer_updates: u64,
    pub rectangles: u64,
    pub raw_bytes: u64,
    pub frames_enqueued: u64,
    pub frames_dropped: u64,
}

#[derive(Default)]
struct RfbTelemetry {
    framebuffer_updates: AtomicU64,
    rectangles: AtomicU64,
    raw_bytes: AtomicU64,
    frames_enqueued: AtomicU64,
    frames_dropped: AtomicU64,
}

impl RfbTelemetry {
    fn snapshot(&self) -> RfbTelemetrySnapshot {
        RfbTelemetrySnapshot {
            framebuffer_updates: self.framebuffer_updates.load(Ordering::Relaxed),
            rectangles: self.rectangles.load(Ordering::Relaxed),
            raw_bytes: self.raw_bytes.load(Ordering::Relaxed),
            frames_enqueued: self.frames_enqueued.load(Ordering::Relaxed),
            frames_dropped: self.frames_dropped.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RfbClientMessage {
    Key { keysym: u32, down: bool },
    Pointer { button_mask: u8, x: u16, y: u16 },
    Refresh { width: u16, height: u16, incremental: bool },
}

pub struct RfbSession {
    pub initial_width: u16,
    pub initial_height: u16,
    pub frame_receiver: Receiver<FrameSnapshot>,
    input_sender: Sender<RfbClientMessage>,
    telemetry: Arc<RfbTelemetry>,
}

impl RfbSession {
    pub fn send(&self, message: RfbClientMessage) -> Result<(), String> {
        self.input_sender
            .send(message)
            .map_err(|error| error.to_string())
    }

    pub fn telemetry_snapshot(&self) -> RfbTelemetrySnapshot {
        self.telemetry.snapshot()
    }
}

pub struct RfbClientTool;

impl RfbClientTool {
    pub fn connect(endpoint: SocketAddr) -> Result<RfbSession, String> {
        if !endpoint.ip().is_loopback() {
            return Err(String::from("RFB endpoint must be loopback"));
        }
        let mut stream = TcpStream::connect_timeout(&endpoint, CONNECT_TIMEOUT)
            .map_err(|error| format!("RFB connection failed: {error}"))?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;

        let (width, height) = perform_handshake(&mut stream)?;
        send_set_pixel_format(&mut stream)?;
        send_set_encodings(&mut stream)?;
        send_framebuffer_request(&mut stream, width, height, false)?;
        stream
            .set_read_timeout(None)
            .map_err(|error| error.to_string())?;

        let reader = stream.try_clone().map_err(|error| error.to_string())?;
        let writer = stream;
        let (frame_sender, frame_receiver) = mpsc::sync_channel(FRAME_QUEUE_DEPTH);
        let (input_sender, input_receiver) = mpsc::channel();
        let refresh_sender = input_sender.clone();
        let telemetry = Arc::new(RfbTelemetry::default());
        let reader_telemetry = Arc::clone(&telemetry);

        thread::Builder::new()
            .name(String::from("turkuaz-rfb-writer"))
            .spawn(move || writer_loop(writer, input_receiver))
            .map_err(|error| error.to_string())?;
        thread::Builder::new()
            .name(String::from("turkuaz-rfb-reader"))
            .spawn(move || {
                reader_loop(
                    reader,
                    width,
                    height,
                    frame_sender,
                    refresh_sender,
                    reader_telemetry,
                )
            })
            .map_err(|error| error.to_string())?;

        Ok(RfbSession {
            initial_width: width,
            initial_height: height,
            frame_receiver,
            input_sender,
            telemetry,
        })
    }
}

fn perform_handshake(stream: &mut TcpStream) -> Result<(u16, u16), String> {
    let mut server_version = [0_u8; 12];
    stream
        .read_exact(&mut server_version)
        .map_err(|error| format!("RFB version read failed: {error}"))?;
    if &server_version[..4] != b"RFB " || server_version[11] != b'\n' {
        return Err(String::from("Invalid RFB server version banner"));
    }
    let minor = std::str::from_utf8(&server_version[8..11])
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| String::from("Unsupported RFB version banner"))?;

    let protocol_version = RfbProtocolVersion::negotiate(minor);
    stream
        .write_all(protocol_version.banner())
        .map_err(|error| format!("RFB version write failed: {error}"))?;

    if protocol_version == RfbProtocolVersion::V3_3 {
        let security = read_u32(stream)?;
        if security == 0 {
            return Err(read_failure_reason(stream)
                .unwrap_or_else(|_| String::from("RFB 3.3 security negotiation failed")));
        }
        if security != u32::from(SECURITY_NONE) {
            return Err(format!("RFB 3.3 security type {security} is unsupported"));
        }
    } else {
        let count = read_u8(stream)?;
        if count == 0 {
            if protocol_version == RfbProtocolVersion::V3_8 {
                return Err(read_failure_reason(stream).unwrap_or_else(|_| {
                    String::from("RFB server offered no security types")
                }));
            }
            return Err(String::from("RFB 3.7 server offered no security types"));
        }
        let mut types = vec![0_u8; usize::from(count)];
        stream
            .read_exact(&mut types)
            .map_err(|error| format!("RFB security list read failed: {error}"))?;
        if !types.contains(&SECURITY_NONE) {
            return Err(String::from(
                "QEMU RFB server does not offer loopback None authentication",
            ));
        }
        stream
            .write_all(&[SECURITY_NONE])
            .map_err(|error| format!("RFB security selection failed: {error}"))?;
        if protocol_version == RfbProtocolVersion::V3_8 {
            let result = read_u32(stream)?;
            if result != 0 {
                return Err(read_failure_reason(stream)
                    .unwrap_or_else(|_| format!("RFB security failed with code {result}")));
            }
        }
    }

    stream
        .write_all(&[CLIENT_SHARED])
        .map_err(|error| format!("RFB ClientInit failed: {error}"))?;
    let width = read_u16(stream)?;
    let height = read_u16(stream)?;
    let mut pixel_format = [0_u8; 16];
    stream
        .read_exact(&mut pixel_format)
        .map_err(|error| format!("RFB ServerInit pixel format failed: {error}"))?;
    let name_length = usize::try_from(read_u32(stream)?).map_err(|_| {
        String::from("RFB server name length cannot be represented on this platform")
    })?;
    if name_length > MAX_SERVER_NAME_BYTES {
        return Err(String::from("RFB server name exceeds safety limit"));
    }
    let mut name = vec![0_u8; name_length];
    stream
        .read_exact(&mut name)
        .map_err(|error| format!("RFB server name read failed: {error}"))?;
    if width == 0 || height == 0 {
        return Err(String::from("RFB framebuffer has invalid zero dimension"));
    }
    Ok((width, height))
}

fn send_set_pixel_format(stream: &mut TcpStream) -> Result<(), String> {
    let mut message = Vec::with_capacity(20);
    message.extend_from_slice(&[MESSAGE_SET_PIXEL_FORMAT, 0, 0, 0]);
    message.extend_from_slice(&[32, 24, 0, 1]);
    message.extend_from_slice(&255_u16.to_be_bytes());
    message.extend_from_slice(&255_u16.to_be_bytes());
    message.extend_from_slice(&255_u16.to_be_bytes());
    message.extend_from_slice(&[16, 8, 0, 0, 0, 0]);
    stream
        .write_all(&message)
        .map_err(|error| format!("RFB SetPixelFormat failed: {error}"))
}

fn send_set_encodings(stream: &mut TcpStream) -> Result<(), String> {
    let encodings = [ENCODING_RAW, ENCODING_DESKTOP_SIZE];
    let mut message = Vec::with_capacity(4 + encodings.len() * 4);
    message.extend_from_slice(&[MESSAGE_SET_ENCODINGS, 0]);
    message.extend_from_slice(&(encodings.len() as u16).to_be_bytes());
    for encoding in encodings {
        message.extend_from_slice(&encoding.to_be_bytes());
    }
    stream
        .write_all(&message)
        .map_err(|error| format!("RFB SetEncodings failed: {error}"))
}

fn writer_loop(mut stream: TcpStream, receiver: Receiver<RfbClientMessage>) {
    while let Ok(message) = receiver.recv() {
        let result = match message {
            RfbClientMessage::Key { keysym, down } => send_key(&mut stream, keysym, down),
            RfbClientMessage::Pointer { button_mask, x, y } => {
                send_pointer(&mut stream, button_mask, x, y)
            }
            RfbClientMessage::Refresh {
                width,
                height,
                incremental,
            } => send_framebuffer_request(&mut stream, width, height, incremental),
        };
        if result.is_err() {
            break;
        }
    }
}

fn reader_loop(
    mut stream: TcpStream,
    initial_width: u16,
    initial_height: u16,
    frame_sender: SyncSender<FrameSnapshot>,
    refresh_sender: Sender<RfbClientMessage>,
    telemetry: Arc<RfbTelemetry>,
) {
    let mut width = initial_width;
    let mut height = initial_height;
    let mut pixels = vec![0_u32; usize::from(width) * usize::from(height)];

    loop {
        let Ok(message_type) = read_u8(&mut stream) else {
            break;
        };
        let is_framebuffer_update = message_type == SERVER_FRAMEBUFFER_UPDATE;
        let result = match message_type {
            SERVER_FRAMEBUFFER_UPDATE => {
                telemetry
                    .framebuffer_updates
                    .fetch_add(1, Ordering::Relaxed);
                read_framebuffer_update(
                    &mut stream,
                    &mut width,
                    &mut height,
                    &mut pixels,
                    &telemetry,
                )
            },
            SERVER_SET_COLOR_MAP => skip_color_map(&mut stream),
            SERVER_BELL => Ok(false),
            SERVER_CUT_TEXT => skip_cut_text(&mut stream),
            other => Err(format!("Unsupported RFB server message: {other}")),
        };
        let Ok(frame_changed) = result else {
            break;
        };
        if frame_changed {
            let frame = FrameSnapshot {
                width,
                height,
                pixels: pixels.clone(),
            };
            match frame_sender.try_send(frame) {
                Ok(()) => {
                    telemetry.frames_enqueued.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Full(_)) => {
                    telemetry.frames_dropped.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Disconnected(_)) => break,
            }
        }
        if is_framebuffer_update {
            let _ = refresh_sender.send(RfbClientMessage::Refresh {
                width,
                height,
                incremental: true,
            });
        }
    }
}

fn read_framebuffer_update(
    stream: &mut TcpStream,
    width: &mut u16,
    height: &mut u16,
    pixels: &mut Vec<u32>,
    telemetry: &RfbTelemetry,
) -> Result<bool, String> {
    let _padding = read_u8(stream)?;
    let rectangles = read_u16(stream)?;
    let mut changed = false;
    for _ in 0..rectangles {
        telemetry.rectangles.fetch_add(1, Ordering::Relaxed);
        let x = read_u16(stream)?;
        let y = read_u16(stream)?;
        let rect_width = read_u16(stream)?;
        let rect_height = read_u16(stream)?;
        let encoding = read_i32(stream)?;
        match encoding {
            ENCODING_RAW => {
                let raw_bytes = read_raw_rectangle(
                    stream,
                    *width,
                    *height,
                    pixels,
                    x,
                    y,
                    rect_width,
                    rect_height,
                )?;
                telemetry.raw_bytes.fetch_add(raw_bytes, Ordering::Relaxed);
                changed = true;
            }
            ENCODING_DESKTOP_SIZE => {
                if rect_width == 0 || rect_height == 0 {
                    return Err(String::from("RFB DesktopSize has zero dimension"));
                }
                *width = rect_width;
                *height = rect_height;
                *pixels = vec![0_u32; usize::from(*width) * usize::from(*height)];
                changed = true;
            }
            other => return Err(format!("Unexpected RFB encoding: {other}")),
        }
    }
    Ok(changed)
}

fn read_raw_rectangle(
    stream: &mut TcpStream,
    framebuffer_width: u16,
    framebuffer_height: u16,
    pixels: &mut [u32],
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> Result<u64, String> {
    let end_x = x.checked_add(width).ok_or_else(|| String::from("RFB rectangle overflow"))?;
    let end_y = y.checked_add(height).ok_or_else(|| String::from("RFB rectangle overflow"))?;
    if end_x > framebuffer_width || end_y > framebuffer_height {
        return Err(String::from("RFB rectangle exceeds framebuffer bounds"));
    }
    let byte_count = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| String::from("RFB raw rectangle size overflow"))?;
    let mut raw = vec![0_u8; byte_count];
    stream
        .read_exact(&mut raw)
        .map_err(|error| format!("RFB raw rectangle read failed: {error}"))?;

    for row in 0..usize::from(height) {
        for column in 0..usize::from(width) {
            let source = (row * usize::from(width) + column) * 4;
            let blue = u32::from(raw[source]);
            let green = u32::from(raw[source + 1]);
            let red = u32::from(raw[source + 2]);
            let target_x = usize::from(x) + column;
            let target_y = usize::from(y) + row;
            let target = target_y * usize::from(framebuffer_width) + target_x;
            pixels[target] = (red << 16) | (green << 8) | blue;
        }
    }
    u64::try_from(byte_count).map_err(|_| String::from("RFB raw byte telemetry overflow"))
}

fn skip_color_map(stream: &mut TcpStream) -> Result<bool, String> {
    let _padding = read_u8(stream)?;
    let _first_color = read_u16(stream)?;
    let color_count = usize::from(read_u16(stream)?);
    let mut data = vec![0_u8; color_count.saturating_mul(6)];
    stream
        .read_exact(&mut data)
        .map_err(|error| format!("RFB color map read failed: {error}"))?;
    Ok(false)
}

fn skip_cut_text(stream: &mut TcpStream) -> Result<bool, String> {
    let mut padding = [0_u8; 3];
    stream
        .read_exact(&mut padding)
        .map_err(|error| format!("RFB cut text padding failed: {error}"))?;
    let length = usize::try_from(read_u32(stream)?)
        .map_err(|_| String::from("RFB cut text length overflow"))?;
    if length > MAX_CUT_TEXT_BYTES {
        return Err(String::from("RFB cut text exceeds safety limit"));
    }
    let mut data = vec![0_u8; length];
    stream
        .read_exact(&mut data)
        .map_err(|error| format!("RFB cut text read failed: {error}"))?;
    Ok(false)
}

fn send_framebuffer_request(
    stream: &mut TcpStream,
    width: u16,
    height: u16,
    incremental: bool,
) -> Result<(), String> {
    let mut message = Vec::with_capacity(10);
    message.push(MESSAGE_FRAMEBUFFER_UPDATE_REQUEST);
    message.push(u8::from(incremental));
    message.extend_from_slice(&0_u16.to_be_bytes());
    message.extend_from_slice(&0_u16.to_be_bytes());
    message.extend_from_slice(&width.to_be_bytes());
    message.extend_from_slice(&height.to_be_bytes());
    stream
        .write_all(&message)
        .map_err(|error| format!("RFB framebuffer request failed: {error}"))
}

fn send_key(stream: &mut TcpStream, keysym: u32, down: bool) -> Result<(), String> {
    let mut message = Vec::with_capacity(8);
    message.push(MESSAGE_KEY_EVENT);
    message.push(u8::from(down));
    message.extend_from_slice(&[0, 0]);
    message.extend_from_slice(&keysym.to_be_bytes());
    stream
        .write_all(&message)
        .map_err(|error| format!("RFB key event failed: {error}"))
}

fn send_pointer(
    stream: &mut TcpStream,
    button_mask: u8,
    x: u16,
    y: u16,
) -> Result<(), String> {
    let mut message = Vec::with_capacity(6);
    message.push(MESSAGE_POINTER_EVENT);
    message.push(button_mask);
    message.extend_from_slice(&x.to_be_bytes());
    message.extend_from_slice(&y.to_be_bytes());
    stream
        .write_all(&message)
        .map_err(|error| format!("RFB pointer event failed: {error}"))
}

fn read_failure_reason(stream: &mut TcpStream) -> Result<String, String> {
    let length = usize::try_from(read_u32(stream)?)
        .map_err(|_| String::from("RFB failure reason length overflow"))?;
    if length > MAX_SERVER_NAME_BYTES {
        return Err(String::from("RFB failure reason exceeds safety limit"));
    }
    let mut data = vec![0_u8; length];
    stream
        .read_exact(&mut data)
        .map_err(|error| format!("RFB failure reason read failed: {error}"))?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn read_u8(stream: &mut TcpStream) -> Result<u8, String> {
    let mut value = [0_u8; 1];
    stream.read_exact(&mut value).map_err(|error| error.to_string())?;
    Ok(value[0])
}

fn read_u16(stream: &mut TcpStream) -> Result<u16, String> {
    let mut value = [0_u8; 2];
    stream.read_exact(&mut value).map_err(|error| error.to_string())?;
    Ok(u16::from_be_bytes(value))
}

fn read_u32(stream: &mut TcpStream) -> Result<u32, String> {
    let mut value = [0_u8; 4];
    stream.read_exact(&mut value).map_err(|error| error.to_string())?;
    Ok(u32::from_be_bytes(value))
}

fn read_i32(stream: &mut TcpStream) -> Result<i32, String> {
    let mut value = [0_u8; 4];
    stream.read_exact(&mut value).map_err(|error| error.to_string())?;
    Ok(i32::from_be_bytes(value))
}

#[cfg(test)]
mod tests {
    use super::RfbProtocolVersion;

    #[test]
    fn protocol_negotiation_never_requests_version_above_server() {
        assert_eq!(RfbProtocolVersion::negotiate(3), RfbProtocolVersion::V3_3);
        assert_eq!(RfbProtocolVersion::negotiate(7), RfbProtocolVersion::V3_7);
        assert_eq!(RfbProtocolVersion::negotiate(8), RfbProtocolVersion::V3_8);
    }

    #[test]
    fn unknown_protocol_minor_falls_back_to_3_3() {
        assert_eq!(RfbProtocolVersion::negotiate(5), RfbProtocolVersion::V3_3);
        assert_eq!(RfbProtocolVersion::negotiate(9), RfbProtocolVersion::V3_3);
    }
}
