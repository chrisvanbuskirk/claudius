use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Global research state for coordinating cancellation and preventing concurrent operations
#[derive(Debug, Clone)]
pub struct ResearchState {
    pub is_running: bool,
    pub cancellation_token: Arc<AtomicBool>,
    pub current_phase: String,
    pub started_at: Option<SystemTime>,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            is_running: false,
            cancellation_token: Arc::new(AtomicBool::new(false)),
            current_phase: String::new(),
            started_at: None,
        }
    }
}

lazy_static! {
    static ref GLOBAL_STATE: Arc<Mutex<ResearchState>> = Arc::new(Mutex::new(ResearchState::default()));
}

/// Get a clone of the current global research state
pub fn get_state() -> ResearchState {
    GLOBAL_STATE
        .lock()
        .expect("Failed to lock research state")
        .clone()
}

/// Check if research is currently running
pub fn is_running() -> bool {
    get_state().is_running
}

/// Set research as running and return the cancellation token
pub fn set_running(phase: &str) -> Result<Arc<AtomicBool>, String> {
    let mut state = GLOBAL_STATE
        .lock()
        .map_err(|e| format!("Failed to lock research state: {}", e))?;

    if state.is_running {
        return Err("Research is already running".to_string());
    }

    // Create new cancellation token
    state.cancellation_token = Arc::new(AtomicBool::new(false));
    state.is_running = true;
    state.current_phase = phase.to_string();
    state.started_at = Some(SystemTime::now());

    Ok(state.cancellation_token.clone())
}

/// Set research as not running
pub fn set_stopped() {
    if let Ok(mut state) = GLOBAL_STATE.lock() {
        state.is_running = false;
        state.current_phase = String::new();
        state.started_at = None;
    }
}

/// Update the current phase
pub fn set_phase(phase: &str) {
    if let Ok(mut state) = GLOBAL_STATE.lock() {
        state.current_phase = phase.to_string();
    }
}

/// Request cancellation of the current research
pub fn cancel() -> Result<(), String> {
    let state = GLOBAL_STATE
        .lock()
        .map_err(|e| format!("Failed to lock research state: {}", e))?;

    if !state.is_running {
        return Err("No research is currently running".to_string());
    }

    state.cancellation_token.store(true, Ordering::Relaxed);
    Ok(())
}

/// Check if cancellation has been requested
pub fn is_cancelled() -> bool {
    get_state().cancellation_token.load(Ordering::Relaxed)
}

/// Reset the global research state (for recovery from errors)
pub fn reset() {
    if let Ok(mut state) = GLOBAL_STATE.lock() {
        *state = ResearchState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        reset();
        assert!(!is_running());
        assert!(!is_cancelled());
    }

    #[test]
    fn test_set_running() {
        reset();
        let token = set_running("starting").unwrap();
        assert!(is_running());
        assert!(!token.load(Ordering::Relaxed));
    }

    #[test]
    fn test_prevent_concurrent_research() {
        reset();
        let _ = set_running("starting").unwrap();
        let result = set_running("starting");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Research is already running");
    }

    #[test]
    fn test_cancellation() {
        reset();
        let _ = set_running("starting").unwrap();
        assert!(!is_cancelled());

        cancel().unwrap();
        assert!(is_cancelled());
    }

    #[test]
    fn test_set_stopped() {
        reset();
        let _ = set_running("starting").unwrap();
        set_stopped();
        assert!(!is_running());
    }

    #[test]
    fn test_phase_updates() {
        reset();
        let _ = set_running("starting").unwrap();
        set_phase("researching");
        assert_eq!(get_state().current_phase, "researching");
    }
}
