// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/vm_state.rs
// # 📌 Amac: VM yasam dongusu state machine kurallarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gecerli VM durumlarini ve izin verilen state gecislerini merkezi olarak modeller
// # Bagimli Oldugu Katman: Service

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Created,
    Stopped,
    Starting,
    Running,
    Pausing,
    Paused,
    Resuming,
    Stopping,
    Error,
}

impl VmState {
    pub const fn can_transition_to(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Created, Self::Stopped)
                | (Self::Stopped, Self::Starting)
                | (Self::Starting, Self::Running)
                | (Self::Starting, Self::Error)
                | (Self::Running, Self::Pausing)
                | (Self::Running, Self::Stopping)
                | (Self::Running, Self::Error)
                | (Self::Pausing, Self::Paused)
                | (Self::Pausing, Self::Error)
                | (Self::Paused, Self::Resuming)
                | (Self::Paused, Self::Stopping)
                | (Self::Paused, Self::Error)
                | (Self::Resuming, Self::Running)
                | (Self::Resuming, Self::Error)
                | (Self::Stopping, Self::Stopped)
                | (Self::Stopping, Self::Error)
                | (Self::Error, Self::Stopped)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_can_start() {
        assert!(VmState::Stopped.can_transition_to(VmState::Starting));
    }

    #[test]
    fn stopped_cannot_jump_to_running() {
        assert!(!VmState::Stopped.can_transition_to(VmState::Running));
    }

    #[test]
    fn error_can_be_recovered_to_stopped() {
        assert!(VmState::Error.can_transition_to(VmState::Stopped));
    }
}
