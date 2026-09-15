// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/gaming_input_bridge_tool.rs
// # 📌 Amac: TurkuazDisplay raw input eventlerini local Engine Gaming Input API'ye non-blocking kuyrukla aktarir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Native event loop'u network I/O'dan ayirir; bounded queue ve local newline-JSON Engine API client worker kullanir
// # Bagimli Oldugu Katman: Service | Tool

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Duration;

use turkuazvm_gaming_input::domain::event::GamingInputEvent;
use turkuazvm_engine_api::{
    EngineAction, EngineRequest, EngineResponse, GamingInputEventDto, ENGINE_API_VERSION,
};

const INPUT_QUEUE_CAPACITY: usize = 256;
const INPUT_REQUEST_TIMEOUT: Duration = Duration::from_millis(350);

#[derive(Clone)]
pub struct GamingInputBridgeTool {
    sender: SyncSender<GamingInputEventDto>,
}

impl GamingInputBridgeTool {
    pub fn spawn(
        vm_id: String,
        endpoint: SocketAddr,
        auth_token: Option<String>,
    ) -> Self {
        let (sender, receiver) = mpsc::sync_channel(INPUT_QUEUE_CAPACITY);
        thread::spawn(move || {
            let mut request_id = 1_u64;
            while let Ok(event) = receiver.recv() {
                let request = EngineRequest {
                    request_id,
                    api_version: ENGINE_API_VERSION,
                    auth_token: auth_token.clone(),
                    action: EngineAction::InjectGamingInputEvent {
                        vm_id: vm_id.clone(),
                        event,
                    },
                };
                request_id = request_id.wrapping_add(1).max(1);
                let _ = send_request(endpoint, &request);
            }
        });
        Self { sender }
    }

    pub fn try_send(&self, event: GamingInputEventDto) {
        let _ = self.sender.try_send(event);
    }

    pub fn try_send_gamepad_event(&self, event: GamingInputEvent) {
        let event = match event {
            GamingInputEvent::GamepadButton { button, pressed } => {
                GamingInputEventDto::GamepadButton { button, pressed }
            }
            GamingInputEvent::GamepadAxis { axis, value_milli } => {
                GamingInputEventDto::GamepadAxis { axis, value_milli }
            }
            _ => return,
        };
        self.try_send(event);
    }

}

fn send_request(endpoint: SocketAddr, request: &EngineRequest) -> Result<(), String> {
    let mut stream = TcpStream::connect_timeout(&endpoint, INPUT_REQUEST_TIMEOUT)
        .map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(INPUT_REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(INPUT_REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    let mut payload = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    payload.push(b'\n');
    stream.write_all(&payload).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
        return Err(String::from("Engine Gaming Input response is empty"));
    }
    let response: EngineResponse = serde_json::from_str(line.trim_end())
        .map_err(|error| error.to_string())?;
    if response.ok {
        Ok(())
    } else {
        Err(response
            .error
            .map(|value| format!("{}: {}", value.code, value.message))
            .unwrap_or_else(|| String::from("Unknown Engine Gaming Input failure")))
    }
}
