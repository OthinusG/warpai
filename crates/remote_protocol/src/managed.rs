//! Validation and receipt-age rules shared by managed SSH status consumers.
use crate::proto::{InputDisposition, ManagedFence};
use uuid::Uuid;
pub const PROTOCOL_MAJOR: u32 = 1;

/// Terminal bytes are bounded and pinned to an admitted process, never local paths.
/// Legacy states omit `guarded`; new states must echo the exact ownership policy.
pub fn valid_terminal_state(state: &crate::proto::TerminalState) -> bool {
    let valid_id = |id: &str| Uuid::parse_str(id).is_ok_and(|id| !id.is_nil());
    let tuple_valid = match state.guarded {
        Some(true) => {
            !state.attachment_id.is_empty()
                && valid_id(&state.attachment_id)
                && state.attachment_generation > 0
        }
        Some(false) => state.attachment_id.is_empty() && state.attachment_generation == 0,
        // Older runtimes do not emit `guarded`; a present tuple must still be coherent.
        None => {
            (state.attachment_id.is_empty() && state.attachment_generation == 0)
                || (valid_id(&state.attachment_id) && state.attachment_generation > 0)
        }
    };
    let label_valid = state.label.as_deref().is_none_or(|label| {
        !label.is_empty() && label.len() <= 128 && !label.chars().any(char::is_control)
    }) && (state.guarded == Some(true) || state.label.is_none());
    let receipt_valid = state.input_receipt.as_ref().is_none_or(|receipt| {
        valid_id(&receipt.receipt_id)
            && receipt.input_sequence > 0
            && receipt.written_bytes <= 4096
            && InputDisposition::try_from(receipt.disposition)
                .is_ok_and(|disposition| disposition != InputDisposition::Unspecified)
            && state.guarded == Some(true)
            && receipt.attachment_generation == state.attachment_generation
            && receipt.input_sequence <= state.accepted_input_sequence
    });
    state.fence.as_ref().is_some_and(|f| valid_fence(f, true))
        && valid_id(&state.session_id)
        && valid_id(&state.run_id)
        && tuple_valid
        && label_valid
        && receipt_valid
        && !(state.processes_active == Some(false) && state.exit_code.is_none())
        && state.output.len() <= 32 * 1024
        && state
            .output_end
            .checked_sub(state.output_offset)
            .is_some_and(|remaining| {
                remaining >= state.output.len() as u64 && remaining <= 256 * 1024
            })
}

/// Nil or malformed IDs never identify a verified attachment.
pub fn valid_fence(fence: &ManagedFence, require_project: bool) -> bool {
    let valid_id = |value: &str| Uuid::parse_str(value).is_ok_and(|id| !id.is_nil());
    valid_id(&fence.service_id)
        && valid_id(&fence.service_boot_id)
        && valid_id(&fence.connection_id)
        && if require_project {
            valid_id(&fence.project_id)
        } else {
            fence.project_id.is_empty()
        }
}

/// Account custody is verified independently from any project attachment.
pub fn valid_account_fence(fence: &crate::proto::AccountFence) -> bool {
    let valid_id = |value: &str| Uuid::parse_str(value).is_ok_and(|id| !id.is_nil());
    valid_id(&fence.guardian_id) && valid_id(&fence.boot_id) && valid_id(&fence.connection_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_fence_requires_exactly_one_verified_connection() {
        let mut fence = crate::proto::AccountFence {
            guardian_id: Uuid::new_v4().to_string(),
            boot_id: Uuid::new_v4().to_string(),
            connection_id: Uuid::new_v4().to_string(),
        };
        assert!(valid_account_fence(&fence));
        fence.boot_id = Uuid::nil().to_string();
        assert!(!valid_account_fence(&fence));
        fence.boot_id = "not-a-uuid".into();
        assert!(!valid_account_fence(&fence));
        fence.boot_id = Uuid::new_v4().to_string();
        fence.connection_id.clear();
        assert!(!valid_account_fence(&fence));
    }

    #[test]
    fn terminal_state_refuses_invalid_ownership_and_unbounded_replay() {
        let mut state = crate::proto::TerminalState {
            fence: Some(ManagedFence {
                service_id: Uuid::new_v4().to_string(),
                service_boot_id: Uuid::new_v4().to_string(),
                connection_id: Uuid::new_v4().to_string(),
                project_id: Uuid::new_v4().to_string(),
            }),
            session_id: Uuid::new_v4().to_string(),
            run_id: Uuid::new_v4().to_string(),
            accepted_input_sequence: 1,
            output_end: 2,
            output: b"ok".to_vec(),
            ..crate::proto::TerminalState::default()
        };
        assert!(valid_terminal_state(&state));
        state.processes_active = Some(false);
        assert!(!valid_terminal_state(&state));
        state.exit_code = Some(0);
        assert!(valid_terminal_state(&state));
        state.exit_code = None;
        state.processes_active = None;
        state.output_end = 1;
        assert!(!valid_terminal_state(&state));
        state.output_end = 2;
        state.output_offset = 3;
        assert!(!valid_terminal_state(&state));
        state.output_offset = 0;
        state.output_end = 256 * 1024 + 1;
        assert!(!valid_terminal_state(&state));
        state.output_end = 64 * 1024;
        state.output = vec![0; 32 * 1024 + 1];
        assert!(!valid_terminal_state(&state));
        state.output = vec![0; 32 * 1024];
        assert!(valid_terminal_state(&state));
        state.run_id = Uuid::nil().to_string();
        assert!(!valid_terminal_state(&state));
        state.run_id = Uuid::new_v4().to_string();
        state.attachment_id = "not-a-uuid".into();
        assert!(!valid_terminal_state(&state));
        state.attachment_id = Uuid::new_v4().to_string();
        state.attachment_generation = 1;
        state.guarded = Some(true);
        assert!(valid_terminal_state(&state));
        state.label = Some("terminal\u{7}fence".into());
        assert!(!valid_terminal_state(&state));
        state.label = Some("build".into());
        assert!(valid_terminal_state(&state));
        state.guarded = Some(false);
        assert!(!valid_terminal_state(&state));
        state.attachment_id.clear();
        state.attachment_generation = 0;
        assert!(!valid_terminal_state(&state));
        state.label = None;
        assert!(valid_terminal_state(&state));
        state.label = Some("build".into());
        assert!(!valid_terminal_state(&state));
        state.label = None;
        state.guarded = Some(true);
        assert!(!valid_terminal_state(&state));
        state.attachment_id = Uuid::new_v4().to_string();
        state.attachment_generation = 1;
        state.input_receipt = Some(crate::proto::InputReceipt {
            attachment_generation: 1,
            input_sequence: 1,
            receipt_id: Uuid::new_v4().to_string(),
            disposition: InputDisposition::InputAccepted as i32,
            written_bytes: 0,
        });
        assert!(valid_terminal_state(&state));
        if let Some(receipt) = &mut state.input_receipt {
            receipt.input_sequence = 0;
        }
        assert!(!valid_terminal_state(&state));
        if let Some(receipt) = &mut state.input_receipt {
            receipt.input_sequence = 1;
            receipt.attachment_generation = 2;
        }
        assert!(!valid_terminal_state(&state));
        if let Some(receipt) = &mut state.input_receipt {
            receipt.attachment_generation = 1;
            receipt.written_bytes = 4097;
        }
        assert!(!valid_terminal_state(&state));
        if let Some(receipt) = &mut state.input_receipt {
            receipt.written_bytes = 4096;
            receipt.receipt_id = Uuid::nil().to_string();
        }
        assert!(!valid_terminal_state(&state));
        if let Some(receipt) = &mut state.input_receipt {
            receipt.receipt_id = Uuid::new_v4().to_string();
        }
        assert!(valid_terminal_state(&state));
        state.accepted_input_sequence = 0;
        assert!(!valid_terminal_state(&state));
        state.accepted_input_sequence = 1;
        assert!(valid_terminal_state(&state));
        state.guarded = None;
        state.input_receipt = None;
        assert!(valid_terminal_state(&state));
        state.attachment_generation = 0;
        assert!(!valid_terminal_state(&state));
        state.attachment_id.clear();
        assert!(valid_terminal_state(&state));
        state.attachment_generation = 1;
        assert!(!valid_terminal_state(&state));
        state.attachment_generation = 0;
        assert!(valid_terminal_state(&state));
    }
}
