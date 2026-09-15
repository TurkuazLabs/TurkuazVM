// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qmp_client_tool.rs
// # 📌 Amac: QEMU Machine Protocol session handshake, introspection ve quit islemlerini yonetir
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Greeting, capabilities, query komutlari, event ayiklama, installer media eject ve process termination kontrolunu uygular
// # Bagimli Oldugu Katman: Service | Tool

use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use turkuazvm_core::domain::hypervisor_control::{
    HypervisorControlReport, HypervisorRuntimeVersion,
};
use turkuazvm_core::ports::hypervisor_monitor_port::{
    HypervisorMonitorError, HypervisorMonitorPort,
};

use crate::domain::qmp::{
    QmpCommandInfo, QmpEventEnvelope, QmpGreetingEnvelope, QmpRequest, QmpResponseEnvelope,
    QmpVersionInfo,
};
use crate::ports::qmp_transport_port::{QmpTransportError, QmpTransportPort};

const COMMAND_QMP_CAPABILITIES: &str = "qmp_capabilities";
const COMMAND_QUERY_VERSION: &str = "query-version";
const COMMAND_QUERY_COMMANDS: &str = "query-commands";
const COMMAND_QUERY_STATUS: &str = "query-status";
const COMMAND_QUIT: &str = "quit";
const COMMAND_EJECT: &str = "eject";
const KEY_EVENT: &str = "event";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QmpSessionState {
    AwaitingGreeting,
    CapabilitiesNegotiated,
    Ready,
}

pub struct QmpClientTool<T>
where
    T: QmpTransportPort,
{
    transport: T,
    state: QmpSessionState,
    pending_events: Vec<String>,
    next_request_id: u64,
}

impl<T> QmpClientTool<T>
where
    T: QmpTransportPort,
{
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            state: QmpSessionState::AwaitingGreeting,
            pending_events: Vec::new(),
            next_request_id: 1,
        }
    }

    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn into_transport(self) -> T {
        self.transport
    }

    pub fn heartbeat(&mut self) -> Result<Vec<String>, HypervisorMonitorError> {
        self.negotiate()?;
        let _ = self.execute_command(COMMAND_QUERY_STATUS)?;
        Ok(self.drain_events())
    }

    pub fn request_quit(&mut self) -> Result<(), HypervisorMonitorError> {
        self.negotiate()?;
        match self.execute_empty_command(COMMAND_QUIT) {
            Ok(()) | Err(HypervisorMonitorError::ConnectionClosed) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn request_media_eject(&mut self, block_device: &str) -> Result<(), HypervisorMonitorError> {
        self.negotiate()?;
        let response_value = self.execute_command_with_arguments(
            COMMAND_EJECT,
            Some(json!({
                "device": block_device,
                "force": true,
            })),
        )?;
        if !response_value.is_object() {
            return Err(HypervisorMonitorError::ProtocolViolation(String::from(
                "QMP eject returned non-object success payload",
            )));
        }
        Ok(())
    }

    fn negotiate(&mut self) -> Result<(), HypervisorMonitorError> {
        if self.state != QmpSessionState::AwaitingGreeting {
            return Ok(());
        }

        let greeting_message = self.read_transport_message()?;
        let greeting: QmpGreetingEnvelope = serde_json::from_str(&greeting_message)
            .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))?;

        let _advertised_capabilities = greeting.qmp.capabilities;
        let _greeting_version = greeting.qmp.version;

        self.execute_empty_command(COMMAND_QMP_CAPABILITIES)?;
        self.state = QmpSessionState::CapabilitiesNegotiated;

        Ok(())
    }

    fn query_version(&mut self) -> Result<QmpVersionInfo, HypervisorMonitorError> {
        self.execute_typed_command(COMMAND_QUERY_VERSION)
    }

    fn query_commands(&mut self) -> Result<Vec<QmpCommandInfo>, HypervisorMonitorError> {
        self.execute_typed_command(COMMAND_QUERY_COMMANDS)
    }

    fn execute_empty_command(
        &mut self,
        command: &'static str,
    ) -> Result<(), HypervisorMonitorError> {
        let response_value = self.execute_command(command)?;

        if !response_value.is_object() {
            return Err(HypervisorMonitorError::ProtocolViolation(format!(
                "QMP command {command} returned non-object success payload"
            )));
        }

        Ok(())
    }

    fn execute_typed_command<R>(
        &mut self,
        command: &'static str,
    ) -> Result<R, HypervisorMonitorError>
    where
        R: DeserializeOwned,
    {
        let value = self.execute_command(command)?;
        serde_json::from_value(value)
            .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))
    }

    fn execute_command(
        &mut self,
        command: &'static str,
    ) -> Result<Value, HypervisorMonitorError> {
        self.execute_command_with_arguments(command, None)
    }

    fn execute_command_with_arguments(
        &mut self,
        command: &'static str,
        arguments: Option<Value>,
    ) -> Result<Value, HypervisorMonitorError> {
        let request_id = self.allocate_request_id();
        let request = QmpRequest {
            execute: command,
            arguments,
            id: request_id,
        };
        let payload = serde_json::to_string(&request)
            .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))?;

        self.transport
            .write_message(&payload)
            .map_err(Self::map_transport_error)?;

        self.read_response_for(request_id)
    }


    fn allocate_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        request_id
    }

    fn read_response_for(&mut self, request_id: u64) -> Result<Value, HypervisorMonitorError> {
        loop {
            let message = self.read_transport_message()?;
            let value: Value = serde_json::from_str(&message)
                .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))?;

            if value.get(KEY_EVENT).is_some() {
                let event: QmpEventEnvelope = serde_json::from_value(value)
                    .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))?;
                self.pending_events.push(event.event);
                continue;
            }

            let response: QmpResponseEnvelope = serde_json::from_value(value)
                .map_err(|error| HypervisorMonitorError::ProtocolViolation(error.to_string()))?;

            if response.id != Some(request_id) {
                if response.id.is_none() {
                    if let Some(error) = response.error {
                        return Err(HypervisorMonitorError::CommandFailed {
                            class: error.class,
                            description: error.desc,
                        });
                    }
                }
                continue;
            }

            if let Some(error) = response.error {
                return Err(HypervisorMonitorError::CommandFailed {
                    class: error.class,
                    description: error.desc,
                });
            }

            return response.return_value.ok_or_else(|| {
                HypervisorMonitorError::ProtocolViolation(format!(
                    "QMP response {request_id} has no return or error payload"
                ))
            });
        }
    }

    fn read_transport_message(&mut self) -> Result<String, HypervisorMonitorError> {
        self.transport
            .read_message()
            .map_err(Self::map_transport_error)
    }

    fn map_transport_error(error: QmpTransportError) -> HypervisorMonitorError {
        match error {
            QmpTransportError::ConnectFailed(message) => {
                HypervisorMonitorError::ConnectionFailed(message)
            }
            QmpTransportError::ReadFailed(message) | QmpTransportError::WriteFailed(message) => {
                HypervisorMonitorError::TransportFailed(message)
            }
            QmpTransportError::ConnectionClosed => HypervisorMonitorError::ConnectionClosed,
        }
    }
}

impl<T> HypervisorMonitorPort for QmpClientTool<T>
where
    T: QmpTransportPort,
{
    fn inspect_control_plane(
        &mut self,
    ) -> Result<HypervisorControlReport, HypervisorMonitorError> {
        self.negotiate()?;
        let version = self.query_version()?;
        let mut commands = self
            .query_commands()?
            .into_iter()
            .map(|command| command.name)
            .collect::<Vec<_>>();
        commands.sort_unstable();
        self.state = QmpSessionState::Ready;

        Ok(HypervisorControlReport {
            version: HypervisorRuntimeVersion {
                major: version.qemu.major,
                minor: version.qemu.minor,
                micro: version.qemu.micro,
                package: version.package,
            },
            supported_commands: commands,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    struct FakeTransport {
        reads: VecDeque<Result<String, QmpTransportError>>,
        writes: Vec<String>,
    }

    impl FakeTransport {
        fn with_messages(messages: &[&str]) -> Self {
            Self {
                reads: messages
                    .iter()
                    .map(|message| Ok((*message).to_owned()))
                    .collect(),
                writes: Vec::new(),
            }
        }

        fn with_results(results: Vec<Result<String, QmpTransportError>>) -> Self {
            Self {
                reads: results.into(),
                writes: Vec::new(),
            }
        }
    }

    impl QmpTransportPort for FakeTransport {
        fn read_message(&mut self) -> Result<String, QmpTransportError> {
            self.reads
                .pop_front()
                .unwrap_or(Err(QmpTransportError::ConnectionClosed))
        }

        fn write_message(&mut self, message: &str) -> Result<(), QmpTransportError> {
            self.writes.push(message.to_owned());
            Ok(())
        }
    }

    #[test]
    fn handshake_and_introspection_returns_runtime_report() {
        let transport = FakeTransport::with_messages(&[
            r#"{"QMP":{"version":{"qemu":{"major":10,"minor":1,"micro":0},"package":""},"capabilities":[]}}"#,
            r#"{"return":{},"id":1}"#,
            r#"{"return":{"qemu":{"major":10,"minor":1,"micro":0},"package":""},"id":2}"#,
            r#"{"event":"STOP","data":{},"timestamp":{"seconds":1,"microseconds":0}}"#,
            r#"{"return":[{"name":"query-version"},{"name":"query-status"}],"id":3}"#,
        ]);
        let mut client = QmpClientTool::new(transport);

        let report = client
            .inspect_control_plane()
            .expect("QMP inspection should succeed");

        assert_eq!(report.version.major, 10);
        assert_eq!(
            report.supported_commands,
            vec![String::from("query-status"), String::from("query-version")]
        );
        assert_eq!(client.drain_events(), vec![String::from("STOP")]);

        let transport = client.into_transport();
        assert_eq!(transport.writes.len(), 3);
        assert!(transport.writes[0].contains(COMMAND_QMP_CAPABILITIES));
        assert!(transport.writes[1].contains(COMMAND_QUERY_VERSION));
        assert!(transport.writes[2].contains(COMMAND_QUERY_COMMANDS));
    }

    #[test]
    fn qmp_command_error_maps_to_core_error() {
        let transport = FakeTransport::with_messages(&[
            r#"{"QMP":{"version":{"qemu":{"major":10,"minor":1,"micro":0},"package":""},"capabilities":[]}}"#,
            r#"{"error":{"class":"GenericError","desc":"capability failure"},"id":1}"#,
        ]);
        let mut client = QmpClientTool::new(transport);

        let error = client
            .inspect_control_plane()
            .expect_err("QMP command error should be propagated");

        assert_eq!(
            error,
            HypervisorMonitorError::CommandFailed {
                class: String::from("GenericError"),
                description: String::from("capability failure"),
            }
        );
    }

    #[test]
    fn installer_media_eject_sends_forced_block_device_command() {
        let transport = FakeTransport::with_messages(&[
            r#"{"QMP":{"version":{"qemu":{"major":10,"minor":1,"micro":0},"package":""},"capabilities":[]}}"#,
            r#"{"return":{},"id":1}"#,
            r#"{"return":{},"id":2}"#,
        ]);
        let mut client = QmpClientTool::new(transport);

        client
            .request_media_eject("installer")
            .expect("installer media eject should succeed");

        let transport = client.into_transport();
        assert_eq!(transport.writes.len(), 2);
        assert!(transport.writes[1].contains(r#""execute":"eject""#));
        assert!(transport.writes[1].contains(r#""device":"installer""#));
        assert!(transport.writes[1].contains(r#""force":true"#));
    }

    #[test]
    fn quit_accepts_premature_connection_close() {
        let transport = FakeTransport::with_results(vec![
            Ok(String::from(
                r#"{"QMP":{"version":{"qemu":{"major":10,"minor":1,"micro":0},"package":""},"capabilities":[]}}"#,
            )),
            Ok(String::from(r#"{"return":{},"id":1}"#)),
            Err(QmpTransportError::ConnectionClosed),
        ]);
        let mut client = QmpClientTool::new(transport);

        client.request_quit().expect("premature EOF after quit is valid");

        let transport = client.into_transport();
        assert_eq!(transport.writes.len(), 2);
        assert!(transport.writes[1].contains(COMMAND_QUIT));
    }
}
