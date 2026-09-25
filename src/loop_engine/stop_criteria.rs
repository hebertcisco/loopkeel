use crate::loop_engine::state::{LoopState, LoopStatus};

pub fn reached(state: &LoopState, max_iterations: u64, success: bool) -> bool {
    state.iteration >= max_iterations
        || success
        || matches!(state.status, LoopStatus::Completed | LoopStatus::Stopped)
}
