// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/runtime_recovery_service.rs
// # 📌 Amac: Hypervisor event journaling, recovery dedupe ve retry/backoff is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Guest reset journaling ve ProcessExit recovery queue kurallarini merkezi policy ile yonetir
// # Bagimli Oldugu Katman: Repo | Tool

use crate::domain::hypervisor_runtime_event::{HypervisorRuntimeEvent, HypervisorRuntimeEventKind};
use crate::domain::runtime_recovery::{RuntimeJournalEvent, RuntimeJournalEventKind, RuntimeRecoveryRequest};
use crate::domain::virtual_machine::VmId;
use crate::ports::runtime_recovery_repository_port::{RuntimeRecoveryRepositoryError, RuntimeRecoveryRepositoryPort};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRecoveryPolicy {
    pub auto_restart: bool,
    pub retry_delay_ms: u64,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRecoveryServiceError {
    Repository(RuntimeRecoveryRepositoryError),
}

pub struct RuntimeRecoveryService<R>
where
    R: RuntimeRecoveryRepositoryPort,
{
    repository: R,
    policy: RuntimeRecoveryPolicy,
}

impl<R> RuntimeRecoveryService<R>
where
    R: RuntimeRecoveryRepositoryPort,
{
    pub const fn new(repository: R, policy: RuntimeRecoveryPolicy) -> Self {
        Self { repository, policy }
    }

    pub fn record_hypervisor_events(
        &mut self,
        events: &[HypervisorRuntimeEvent],
        observed_at_unix_ms: u64,
    ) -> Result<(), RuntimeRecoveryServiceError> {
        let mut pending = self.repository.pending_requests().map_err(RuntimeRecoveryServiceError::Repository)?;
        for event in events {
            let journal = RuntimeJournalEvent {
                vm_id: event.vm_id.clone(),
                observed_at_unix_ms,
                kind: map_event_kind(&event.kind),
            };
            let sequence = self.repository.append_event(&journal).map_err(RuntimeRecoveryServiceError::Repository)?;
            if self.policy.auto_restart
                && matches!(event.kind, HypervisorRuntimeEventKind::ProcessExited)
                && !pending.iter().any(|request| request.vm_id == event.vm_id)
            {
                let request = RuntimeRecoveryRequest {
                    source_sequence: sequence,
                    vm_id: event.vm_id.clone(),
                    reason: String::from("unexpected_runtime_exit"),
                    attempts: 0,
                    next_attempt_unix_ms: observed_at_unix_ms,
                };
                self.repository.save_request(&request).map_err(RuntimeRecoveryServiceError::Repository)?;
                pending.push(request);
            }
        }
        Ok(())
    }

    pub fn due_requests(&self, now_unix_ms: u64) -> Result<Vec<RuntimeRecoveryRequest>, RuntimeRecoveryServiceError> {
        let mut requests = self.repository.pending_requests().map_err(RuntimeRecoveryServiceError::Repository)?;
        requests.retain(|request| request.next_attempt_unix_ms <= now_unix_ms);
        requests.sort_by_key(|request| (request.next_attempt_unix_ms, request.source_sequence));
        Ok(requests)
    }

    pub fn record_success(
        &mut self,
        request: &RuntimeRecoveryRequest,
        observed_at_unix_ms: u64,
    ) -> Result<(), RuntimeRecoveryServiceError> {
        self.repository.append_event(&RuntimeJournalEvent {
            vm_id: request.vm_id.clone(),
            observed_at_unix_ms,
            kind: RuntimeJournalEventKind::RecoverySucceeded,
        }).map_err(RuntimeRecoveryServiceError::Repository)?;
        self.repository.remove_request(&request.vm_id).map_err(RuntimeRecoveryServiceError::Repository)
    }

    pub fn record_failure(
        &mut self,
        request: &RuntimeRecoveryRequest,
        observed_at_unix_ms: u64,
        detail: String,
    ) -> Result<bool, RuntimeRecoveryServiceError> {
        self.repository.append_event(&RuntimeJournalEvent {
            vm_id: request.vm_id.clone(),
            observed_at_unix_ms,
            kind: RuntimeJournalEventKind::RecoveryFailed { detail },
        }).map_err(RuntimeRecoveryServiceError::Repository)?;
        let attempts = request.attempts.saturating_add(1);
        if attempts >= self.policy.max_attempts {
            self.repository.append_event(&RuntimeJournalEvent {
                vm_id: request.vm_id.clone(),
                observed_at_unix_ms,
                kind: RuntimeJournalEventKind::RecoveryExhausted,
            }).map_err(RuntimeRecoveryServiceError::Repository)?;
            self.repository.remove_request(&request.vm_id).map_err(RuntimeRecoveryServiceError::Repository)?;
            return Ok(false);
        }
        let next_attempt_unix_ms = observed_at_unix_ms.saturating_add(self.policy.retry_delay_ms);
        self.repository.save_request(&RuntimeRecoveryRequest {
            source_sequence: request.source_sequence,
            vm_id: request.vm_id.clone(),
            reason: request.reason.clone(),
            attempts,
            next_attempt_unix_ms,
        }).map_err(RuntimeRecoveryServiceError::Repository)?;
        Ok(true)
    }

    pub fn clear_request(&mut self, vm_id: &VmId) -> Result<(), RuntimeRecoveryServiceError> {
        self.repository.remove_request(vm_id).map_err(RuntimeRecoveryServiceError::Repository)
    }
}

fn map_event_kind(kind: &HypervisorRuntimeEventKind) -> RuntimeJournalEventKind {
    match kind {
        HypervisorRuntimeEventKind::Qmp { name } => RuntimeJournalEventKind::Qmp { name: name.clone() },
        HypervisorRuntimeEventKind::GuestReset => RuntimeJournalEventKind::GuestReset,
        HypervisorRuntimeEventKind::ProcessExited => RuntimeJournalEventKind::ProcessExited,
        HypervisorRuntimeEventKind::ControlReattached => RuntimeJournalEventKind::ControlReattached,
        HypervisorRuntimeEventKind::ControlUnavailable { detail } => RuntimeJournalEventKind::ControlUnavailable { detail: detail.clone() },
    }
}
