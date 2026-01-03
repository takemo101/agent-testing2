//! Core data types for the Pomodoro Timer.
//!
//! This module defines the data structures used for:
//! - Timer state management
//! - Timer configuration with validation
//! - IPC request/response serialization

use serde::{Deserialize, Serialize};

// ============================================================================
// TimerPhase
// ============================================================================

/// Represents the current phase of the timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerPhase {
    /// Timer is stopped
    Stopped,
    /// Currently in a work session
    Working,
    /// Currently in a short break
    Breaking,
    /// Currently in a long break (after 4 pomodoros)
    LongBreaking,
    /// Timer is paused
    Paused,
}

impl TimerPhase {
    /// Returns the string representation of the phase.
    pub fn as_str(&self) -> &'static str {
        match self {
            TimerPhase::Stopped => "stopped",
            TimerPhase::Working => "working",
            TimerPhase::Breaking => "breaking",
            TimerPhase::LongBreaking => "long_breaking",
            TimerPhase::Paused => "paused",
        }
    }

    /// Returns true if the timer is actively counting down.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        )
    }
}

impl Default for TimerPhase {
    fn default() -> Self {
        TimerPhase::Stopped
    }
}

// ============================================================================
// PomodoroConfig
// ============================================================================

/// Configuration for the Pomodoro timer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PomodoroConfig {
    /// Work duration in minutes (1-120)
    pub work_minutes: u32,
    /// Short break duration in minutes (1-60)
    pub break_minutes: u32,
    /// Long break duration in minutes (1-60)
    pub long_break_minutes: u32,
    /// Whether to automatically start the next cycle
    pub auto_cycle: bool,
    /// Whether to enable Focus Mode integration
    pub focus_mode: bool,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            auto_cycle: false,
            focus_mode: false,
        }
    }
}

impl PomodoroConfig {
    /// Creates a new configuration with the specified work duration.
    pub fn with_work_minutes(mut self, minutes: u32) -> Self {
        self.work_minutes = minutes;
        self
    }

    /// Creates a new configuration with the specified break duration.
    pub fn with_break_minutes(mut self, minutes: u32) -> Self {
        self.break_minutes = minutes;
        self
    }

    /// Creates a new configuration with the specified long break duration.
    pub fn with_long_break_minutes(mut self, minutes: u32) -> Self {
        self.long_break_minutes = minutes;
        self
    }

    /// Validates the configuration.
    ///
    /// Returns an error message if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.work_minutes < 1 || self.work_minutes > 120 {
            return Err("作業時間は1-120分の範囲で指定してください".to_string());
        }
        if self.break_minutes < 1 || self.break_minutes > 60 {
            return Err("休憩時間は1-60分の範囲で指定してください".to_string());
        }
        if self.long_break_minutes < 1 || self.long_break_minutes > 60 {
            return Err("長い休憩時間は1-60分の範囲で指定してください".to_string());
        }
        Ok(())
    }
}

// ============================================================================
// TimerState
// ============================================================================

/// Represents the current state of the timer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    /// Current phase of the timer
    pub phase: TimerPhase,
    /// Remaining seconds in the current phase
    pub remaining_seconds: u32,
    /// Number of completed pomodoros
    pub pomodoro_count: u32,
    /// Current task name (if any)
    pub task_name: Option<String>,
    /// Timer configuration
    pub config: PomodoroConfig,
    /// Previous phase (used for resume after pause)
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_phase: Option<TimerPhase>,
}

impl TimerState {
    /// Creates a new TimerState in stopped state.
    pub fn new(config: PomodoroConfig) -> Self {
        Self {
            phase: TimerPhase::Stopped,
            remaining_seconds: 0,
            pomodoro_count: 0,
            task_name: None,
            config,
            previous_phase: None,
        }
    }

    /// Starts a work session.
    pub fn start_working(&mut self, task_name: Option<String>) {
        self.phase = TimerPhase::Working;
        self.remaining_seconds = self.config.work_minutes * 60;
        self.task_name = task_name;
        self.previous_phase = None;
    }

    /// Starts a break session.
    ///
    /// Automatically chooses between short and long break based on pomodoro count.
    pub fn start_breaking(&mut self) {
        // Long break after every 4 pomodoros
        if self.pomodoro_count > 0 && self.pomodoro_count % 4 == 0 {
            self.phase = TimerPhase::LongBreaking;
            self.remaining_seconds = self.config.long_break_minutes * 60;
        } else {
            self.phase = TimerPhase::Breaking;
            self.remaining_seconds = self.config.break_minutes * 60;
        }
        self.previous_phase = None;
    }

    /// Pauses the timer.
    ///
    /// Only works if timer is currently running.
    pub fn pause(&mut self) {
        if self.phase.is_active() {
            self.previous_phase = Some(self.phase);
            self.phase = TimerPhase::Paused;
        }
    }

    /// Resumes the timer from pause.
    ///
    /// Restores the previous phase before pause.
    pub fn resume(&mut self) {
        if self.phase == TimerPhase::Paused {
            if let Some(prev) = self.previous_phase.take() {
                self.phase = prev;
            } else {
                // Fallback to working if previous phase is unknown
                self.phase = TimerPhase::Working;
            }
        }
    }

    /// Stops the timer and resets to initial state.
    pub fn stop(&mut self) {
        self.phase = TimerPhase::Stopped;
        self.remaining_seconds = 0;
        self.task_name = None;
        self.previous_phase = None;
    }

    /// Decrements the timer by one second.
    ///
    /// Returns true if the timer has completed (reached 0).
    pub fn tick(&mut self) -> bool {
        if self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
        }
        self.remaining_seconds == 0
    }

    /// Returns true if the timer is actively running.
    pub fn is_running(&self) -> bool {
        self.phase.is_active()
    }

    /// Returns true if the timer is paused.
    pub fn is_paused(&self) -> bool {
        self.phase == TimerPhase::Paused
    }

    /// Increments the pomodoro count.
    pub fn increment_pomodoro_count(&mut self) {
        self.pomodoro_count += 1;
    }
}

// ============================================================================
// IPC Types
// ============================================================================

/// Parameters for the start command.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartParams {
    /// Work duration in minutes
    #[serde(rename = "workMinutes", skip_serializing_if = "Option::is_none")]
    pub work_minutes: Option<u32>,
    /// Break duration in minutes
    #[serde(rename = "breakMinutes", skip_serializing_if = "Option::is_none")]
    pub break_minutes: Option<u32>,
    /// Long break duration in minutes
    #[serde(rename = "longBreakMinutes", skip_serializing_if = "Option::is_none")]
    pub long_break_minutes: Option<u32>,
    /// Task name
    #[serde(rename = "taskName", skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
    /// Auto cycle flag
    #[serde(rename = "autoCycle", skip_serializing_if = "Option::is_none")]
    pub auto_cycle: Option<bool>,
    /// Focus mode flag
    #[serde(rename = "focusMode", skip_serializing_if = "Option::is_none")]
    pub focus_mode: Option<bool>,
}

/// IPC request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "lowercase")]
pub enum IpcRequest {
    /// Start a new timer session
    Start {
        /// Start parameters
        #[serde(flatten)]
        params: StartParams,
    },
    /// Pause the current timer
    Pause,
    /// Resume the paused timer
    Resume,
    /// Stop the current timer
    Stop,
    /// Query the current status
    Status,
}

/// Response data for IPC responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseData {
    /// Current state/phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Remaining seconds
    #[serde(rename = "remainingSeconds", skip_serializing_if = "Option::is_none")]
    pub remaining_seconds: Option<u32>,
    /// Completed pomodoro count
    #[serde(rename = "pomodoroCount", skip_serializing_if = "Option::is_none")]
    pub pomodoro_count: Option<u32>,
    /// Current task name
    #[serde(rename = "taskName", skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
}

impl ResponseData {
    /// Creates response data from timer state.
    pub fn from_timer_state(state: &TimerState) -> Self {
        Self {
            state: Some(state.phase.as_str().to_string()),
            remaining_seconds: Some(state.remaining_seconds),
            pomodoro_count: Some(state.pomodoro_count),
            task_name: state.task_name.clone(),
        }
    }
}

/// IPC response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Response status ("success" or "error")
    pub status: String,
    /// Human-readable message
    pub message: String,
    /// Optional response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ResponseData>,
}

impl IpcResponse {
    /// Creates a success response.
    pub fn success(message: impl Into<String>, data: Option<ResponseData>) -> Self {
        Self {
            status: "success".to_string(),
            message: message.into(),
            data,
        }
    }

    /// Creates an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            data: None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // TimerPhase Tests
    // ------------------------------------------------------------------------

    mod timer_phase_tests {
        use super::*;

        #[test]
        fn test_default_is_stopped() {
            assert_eq!(TimerPhase::default(), TimerPhase::Stopped);
        }

        #[test]
        fn test_as_str() {
            assert_eq!(TimerPhase::Stopped.as_str(), "stopped");
            assert_eq!(TimerPhase::Working.as_str(), "working");
            assert_eq!(TimerPhase::Breaking.as_str(), "breaking");
            assert_eq!(TimerPhase::LongBreaking.as_str(), "long_breaking");
            assert_eq!(TimerPhase::Paused.as_str(), "paused");
        }

        #[test]
        fn test_is_active() {
            assert!(!TimerPhase::Stopped.is_active());
            assert!(TimerPhase::Working.is_active());
            assert!(TimerPhase::Breaking.is_active());
            assert!(TimerPhase::LongBreaking.is_active());
            assert!(!TimerPhase::Paused.is_active());
        }

        #[test]
        fn test_serialize_deserialize() {
            let phase = TimerPhase::Working;
            let json = serde_json::to_string(&phase).unwrap();
            assert_eq!(json, "\"working\"");

            let deserialized: TimerPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, TimerPhase::Working);
        }

        #[test]
        fn test_clone_and_copy() {
            let phase = TimerPhase::Breaking;
            let cloned = phase.clone();
            let copied = phase;
            assert_eq!(phase, cloned);
            assert_eq!(phase, copied);
        }
    }

    // ------------------------------------------------------------------------
    // PomodoroConfig Tests
    // ------------------------------------------------------------------------

    mod pomodoro_config_tests {
        use super::*;

        #[test]
        fn test_default_values() {
            let config = PomodoroConfig::default();
            assert_eq!(config.work_minutes, 25);
            assert_eq!(config.break_minutes, 5);
            assert_eq!(config.long_break_minutes, 15);
            assert!(!config.auto_cycle);
            assert!(!config.focus_mode);
        }

        #[test]
        fn test_builder_pattern() {
            let config = PomodoroConfig::default()
                .with_work_minutes(30)
                .with_break_minutes(10)
                .with_long_break_minutes(20);

            assert_eq!(config.work_minutes, 30);
            assert_eq!(config.break_minutes, 10);
            assert_eq!(config.long_break_minutes, 20);
        }

        #[test]
        fn test_validate_success() {
            let config = PomodoroConfig {
                work_minutes: 30,
                break_minutes: 10,
                long_break_minutes: 20,
                auto_cycle: true,
                focus_mode: true,
            };
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_validate_boundary_values() {
            // Minimum valid values
            let config = PomodoroConfig {
                work_minutes: 1,
                break_minutes: 1,
                long_break_minutes: 1,
                auto_cycle: false,
                focus_mode: false,
            };
            assert!(config.validate().is_ok());

            // Maximum valid values
            let config = PomodoroConfig {
                work_minutes: 120,
                break_minutes: 60,
                long_break_minutes: 60,
                auto_cycle: false,
                focus_mode: false,
            };
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_validate_work_minutes_too_low() {
            let config = PomodoroConfig {
                work_minutes: 0,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validate_work_minutes_too_high() {
            let config = PomodoroConfig {
                work_minutes: 121,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validate_break_minutes_too_low() {
            let config = PomodoroConfig {
                break_minutes: 0,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validate_break_minutes_too_high() {
            let config = PomodoroConfig {
                break_minutes: 61,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validate_long_break_minutes_too_low() {
            let config = PomodoroConfig {
                long_break_minutes: 0,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validate_long_break_minutes_too_high() {
            let config = PomodoroConfig {
                long_break_minutes: 61,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_serialize_deserialize() {
            let config = PomodoroConfig {
                work_minutes: 30,
                break_minutes: 10,
                long_break_minutes: 20,
                auto_cycle: true,
                focus_mode: true,
            };

            let json = serde_json::to_string(&config).unwrap();
            let deserialized: PomodoroConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }

    // ------------------------------------------------------------------------
    // TimerState Tests
    // ------------------------------------------------------------------------

    mod timer_state_tests {
        use super::*;

        #[test]
        fn test_new_state() {
            let config = PomodoroConfig::default();
            let state = TimerState::new(config.clone());

            assert_eq!(state.phase, TimerPhase::Stopped);
            assert_eq!(state.remaining_seconds, 0);
            assert_eq!(state.pomodoro_count, 0);
            assert_eq!(state.task_name, None);
            assert_eq!(state.config.work_minutes, config.work_minutes);
        }

        #[test]
        fn test_start_working() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            state.start_working(Some("Test Task".to_string()));

            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.remaining_seconds, 25 * 60);
            assert_eq!(state.task_name, Some("Test Task".to_string()));
        }

        #[test]
        fn test_start_working_no_task() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            state.start_working(None);

            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.remaining_seconds, 25 * 60);
            assert_eq!(state.task_name, None);
        }

        #[test]
        fn test_start_breaking_short() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 1;

            state.start_breaking();

            assert_eq!(state.phase, TimerPhase::Breaking);
            assert_eq!(state.remaining_seconds, 5 * 60);
        }

        #[test]
        fn test_start_breaking_long_after_4_pomodoros() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 4;

            state.start_breaking();

            assert_eq!(state.phase, TimerPhase::LongBreaking);
            assert_eq!(state.remaining_seconds, 15 * 60);
        }

        #[test]
        fn test_start_breaking_long_after_8_pomodoros() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 8;

            state.start_breaking();

            assert_eq!(state.phase, TimerPhase::LongBreaking);
            assert_eq!(state.remaining_seconds, 15 * 60);
        }

        #[test]
        fn test_pause_from_working() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 100;

            state.pause();

            assert_eq!(state.phase, TimerPhase::Paused);
            assert_eq!(state.remaining_seconds, 100);
        }

        #[test]
        fn test_pause_from_breaking() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 1;
            state.start_breaking();

            state.pause();

            assert_eq!(state.phase, TimerPhase::Paused);
        }

        #[test]
        fn test_pause_from_stopped_does_nothing() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            state.pause();

            assert_eq!(state.phase, TimerPhase::Stopped);
        }

        #[test]
        fn test_resume_from_paused_working() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 100;
            state.pause();

            state.resume();

            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.remaining_seconds, 100);
        }

        #[test]
        fn test_resume_from_paused_breaking() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 1;
            state.start_breaking();
            state.pause();

            state.resume();

            assert_eq!(state.phase, TimerPhase::Breaking);
        }

        #[test]
        fn test_resume_from_non_paused_does_nothing() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);

            state.resume();

            assert_eq!(state.phase, TimerPhase::Working);
        }

        #[test]
        fn test_stop() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(Some("Task".to_string()));
            state.remaining_seconds = 500;
            state.pomodoro_count = 3;

            state.stop();

            assert_eq!(state.phase, TimerPhase::Stopped);
            assert_eq!(state.remaining_seconds, 0);
            assert_eq!(state.task_name, None);
            // pomodoro_count is preserved
            assert_eq!(state.pomodoro_count, 3);
        }

        #[test]
        fn test_tick() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 2;

            let completed = state.tick();
            assert!(!completed);
            assert_eq!(state.remaining_seconds, 1);

            let completed = state.tick();
            assert!(completed);
            assert_eq!(state.remaining_seconds, 0);
        }

        #[test]
        fn test_tick_at_zero() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.remaining_seconds = 0;

            let completed = state.tick();
            assert!(completed);
            assert_eq!(state.remaining_seconds, 0);
        }

        #[test]
        fn test_is_running() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            assert!(!state.is_running());

            state.start_working(None);
            assert!(state.is_running());

            state.pause();
            assert!(!state.is_running());

            state.resume();
            assert!(state.is_running());

            state.stop();
            assert!(!state.is_running());
        }

        #[test]
        fn test_is_paused() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            assert!(!state.is_paused());

            state.start_working(None);
            assert!(!state.is_paused());

            state.pause();
            assert!(state.is_paused());

            state.resume();
            assert!(!state.is_paused());
        }

        #[test]
        fn test_increment_pomodoro_count() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);

            assert_eq!(state.pomodoro_count, 0);

            state.increment_pomodoro_count();
            assert_eq!(state.pomodoro_count, 1);

            state.increment_pomodoro_count();
            assert_eq!(state.pomodoro_count, 2);
        }

        #[test]
        fn test_serialize_deserialize() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(Some("Test".to_string()));
            state.remaining_seconds = 1234;
            state.pomodoro_count = 5;

            let json = serde_json::to_string(&state).unwrap();
            let deserialized: TimerState = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.phase, TimerPhase::Working);
            assert_eq!(deserialized.remaining_seconds, 1234);
            assert_eq!(deserialized.pomodoro_count, 5);
            assert_eq!(deserialized.task_name, Some("Test".to_string()));
        }
    }

    // ------------------------------------------------------------------------
    // IPC Types Tests
    // ------------------------------------------------------------------------

    mod ipc_tests {
        use super::*;

        #[test]
        fn test_start_params_default() {
            let params = StartParams::default();
            assert!(params.work_minutes.is_none());
            assert!(params.break_minutes.is_none());
            assert!(params.long_break_minutes.is_none());
            assert!(params.task_name.is_none());
            assert!(params.auto_cycle.is_none());
            assert!(params.focus_mode.is_none());
        }

        #[test]
        fn test_ipc_request_start_serialize() {
            let request = IpcRequest::Start {
                params: StartParams {
                    work_minutes: Some(30),
                    break_minutes: Some(10),
                    long_break_minutes: Some(20),
                    task_name: Some("Test".to_string()),
                    auto_cycle: Some(true),
                    focus_mode: Some(false),
                },
            };

            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"command\":\"start\""));
            assert!(json.contains("\"workMinutes\":30"));
            assert!(json.contains("\"breakMinutes\":10"));
            assert!(json.contains("\"taskName\":\"Test\""));
        }

        #[test]
        fn test_ipc_request_start_deserialize() {
            let json = r#"{"command":"start","workMinutes":25,"taskName":"Coding"}"#;
            let request: IpcRequest = serde_json::from_str(json).unwrap();

            match request {
                IpcRequest::Start { params } => {
                    assert_eq!(params.work_minutes, Some(25));
                    assert_eq!(params.task_name, Some("Coding".to_string()));
                    assert!(params.break_minutes.is_none());
                }
                _ => panic!("Expected Start request"),
            }
        }

        #[test]
        fn test_ipc_request_pause_serialize() {
            let request = IpcRequest::Pause;
            let json = serde_json::to_string(&request).unwrap();
            assert_eq!(json, r#"{"command":"pause"}"#);
        }

        #[test]
        fn test_ipc_request_pause_deserialize() {
            let json = r#"{"command":"pause"}"#;
            let request: IpcRequest = serde_json::from_str(json).unwrap();
            assert!(matches!(request, IpcRequest::Pause));
        }

        #[test]
        fn test_ipc_request_resume_serialize() {
            let request = IpcRequest::Resume;
            let json = serde_json::to_string(&request).unwrap();
            assert_eq!(json, r#"{"command":"resume"}"#);
        }

        #[test]
        fn test_ipc_request_stop_serialize() {
            let request = IpcRequest::Stop;
            let json = serde_json::to_string(&request).unwrap();
            assert_eq!(json, r#"{"command":"stop"}"#);
        }

        #[test]
        fn test_ipc_request_status_serialize() {
            let request = IpcRequest::Status;
            let json = serde_json::to_string(&request).unwrap();
            assert_eq!(json, r#"{"command":"status"}"#);
        }

        #[test]
        fn test_response_data_from_timer_state() {
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(Some("Test Task".to_string()));
            state.remaining_seconds = 1200;
            state.pomodoro_count = 3;

            let data = ResponseData::from_timer_state(&state);

            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.remaining_seconds, Some(1200));
            assert_eq!(data.pomodoro_count, Some(3));
            assert_eq!(data.task_name, Some("Test Task".to_string()));
        }

        #[test]
        fn test_ipc_response_success() {
            let response = IpcResponse::success(
                "Timer started",
                Some(ResponseData {
                    state: Some("working".to_string()),
                    remaining_seconds: Some(1500),
                    pomodoro_count: Some(1),
                    task_name: Some("Test".to_string()),
                }),
            );

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "Timer started");
            assert!(response.data.is_some());

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.remaining_seconds, Some(1500));
        }

        #[test]
        fn test_ipc_response_success_no_data() {
            let response = IpcResponse::success("Paused", None);

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "Paused");
            assert!(response.data.is_none());
        }

        #[test]
        fn test_ipc_response_error() {
            let response = IpcResponse::error("Timer is already running");

            assert_eq!(response.status, "error");
            assert_eq!(response.message, "Timer is already running");
            assert!(response.data.is_none());
        }

        #[test]
        fn test_ipc_response_serialize() {
            let response = IpcResponse::success(
                "OK",
                Some(ResponseData {
                    state: Some("working".to_string()),
                    remaining_seconds: Some(1500),
                    pomodoro_count: Some(1),
                    task_name: None,
                }),
            );

            let json = serde_json::to_string(&response).unwrap();
            assert!(json.contains("\"status\":\"success\""));
            assert!(json.contains("\"remainingSeconds\":1500"));
            // taskName should not be present since it's None
            assert!(!json.contains("taskName"));
        }

        #[test]
        fn test_ipc_response_deserialize() {
            let json = r#"{"status":"success","message":"OK","data":{"state":"working","remainingSeconds":1500}}"#;
            let response: IpcResponse = serde_json::from_str(json).unwrap();

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "OK");
            assert!(response.data.is_some());

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.remaining_seconds, Some(1500));
        }

        #[test]
        fn test_ipc_request_all_commands() {
            // Test all command variants can be deserialized
            let commands = vec![
                (r#"{"command":"start"}"#, "start"),
                (r#"{"command":"pause"}"#, "pause"),
                (r#"{"command":"resume"}"#, "resume"),
                (r#"{"command":"stop"}"#, "stop"),
                (r#"{"command":"status"}"#, "status"),
            ];

            for (json, expected) in commands {
                let request: IpcRequest = serde_json::from_str(json).unwrap();
                match (&request, expected) {
                    (IpcRequest::Start { .. }, "start") => {}
                    (IpcRequest::Pause, "pause") => {}
                    (IpcRequest::Resume, "resume") => {}
                    (IpcRequest::Stop, "stop") => {}
                    (IpcRequest::Status, "status") => {}
                    _ => panic!("Unexpected request type for {}", json),
                }
            }
        }
    }
}
