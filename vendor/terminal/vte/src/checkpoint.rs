use super::{MAX_INTERMEDIATES, MAX_OSC_PARAMS, Params, Parser, State};

/// Portable state at a byte cut, including an unfinished UTF-8 or control sequence.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Checkpoint {
    state: State,
    intermediates: [u8; MAX_INTERMEDIATES],
    intermediate_idx: usize,
    params: Params,
    param: u16,
    osc_raw: Vec<u8>,
    osc_params: [(usize, usize); MAX_OSC_PARAMS],
    osc_num_params: usize,
    ignoring: bool,
    partial_utf8: [u8; 4],
    partial_utf8_len: usize,
}

impl<const OSC_BYTES: usize> Parser<OSC_BYTES> {
    /// Capture parser state without advancing input or invoking a performer.
    pub fn checkpoint(&self) -> Checkpoint {
        let mut partial_utf8 = [0; 4];
        partial_utf8[..self.partial_utf8_len].copy_from_slice(&self.partial_utf8[..self.partial_utf8_len]);
        Checkpoint {
            state: self.state,
            intermediates: self.intermediates,
            intermediate_idx: self.intermediate_idx,
            params: self.params.clone(),
            param: self.param,
            osc_raw: self.osc_raw.clone(),
            osc_params: self.osc_params,
            osc_num_params: self.osc_num_params,
            ignoring: self.ignoring,
            partial_utf8,
            partial_utf8_len: self.partial_utf8_len,
        }
    }

    /// Restore only validated indices and incomplete bytes. No performer is called.
    pub fn from_checkpoint(checkpoint: Checkpoint, max_osc_bytes: usize) -> Option<Self> {
        if checkpoint.intermediate_idx > MAX_INTERMEDIATES
            || !checkpoint.params.valid_checkpoint()
            || checkpoint.osc_num_params > MAX_OSC_PARAMS
            || checkpoint.osc_raw.len() > max_osc_bytes
            || (checkpoint.state != State::OscString
                && (!checkpoint.osc_raw.is_empty() || checkpoint.osc_num_params != 0))
            || checkpoint.partial_utf8_len > 3
        {
            return None;
        }
        let mut previous_end = 0;
        for &(start, end) in &checkpoint.osc_params[..checkpoint.osc_num_params] {
            if start != previous_end || start > end || end > checkpoint.osc_raw.len() {
                return None;
            }
            previous_end = end;
        }
        let partial = &checkpoint.partial_utf8[..checkpoint.partial_utf8_len];
        if !partial.is_empty()
            && !core::str::from_utf8(partial).is_err_and(|error| error.error_len().is_none())
        {
            return None;
        }
        Some(Self {
            state: checkpoint.state,
            intermediates: checkpoint.intermediates,
            intermediate_idx: checkpoint.intermediate_idx,
            params: checkpoint.params,
            param: checkpoint.param,
            osc_raw: checkpoint.osc_raw,
            osc_params: checkpoint.osc_params,
            osc_num_params: checkpoint.osc_num_params,
            ignoring: checkpoint.ignoring,
            partial_utf8: checkpoint.partial_utf8,
            partial_utf8_len: checkpoint.partial_utf8_len,
        })
    }
}
