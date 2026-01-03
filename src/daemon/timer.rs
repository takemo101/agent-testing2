//! Timer engine for the Pomodoro Timer.
//!
//! This module provides the core timer functionality:
//! - State transitions (Working → Breaking → Stopped)
//! - Countdown with tokio::time::interval
//! - Event firing for notifications and sounds
//! - Auto-cycle feature
//! - Long break after 4 pomodoros

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, MissedTickBehavior};

use crate::types::{PomodoroConfig, TimerPhase, TimerState};

// ============================================================================
// TimerEvent
// ============================================================================

/// Timer events for notifications and external integrations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerEvent {
    /// Work session started
    WorkStarted {
        /// Task name (if any)
        task_name: Option<String>,
    },
    /// Work session completed
    WorkCompleted {
        /// Total pomodoro count
        pomodoro_count: u32,
        /// Task name (if any)
        task_name: Option<String>,
    },
    /// Break session started
    BreakStarted {
        /// Whether this is a long break
        is_long_break: bool,
    },
    /// Break session completed
    BreakCompleted {
        /// Whether this was a long break
        is_long_break: bool,
    },
    /// Timer paused
    Paused,
    /// Timer resumed
    Resumed,
    /// Timer stopped
    Stopped,
    /// One second elapsed (tick)
    Tick {
        /// Remaining seconds
        remaining_seconds: u32,
    },
}

// ============================================================================
// TimerEngine
// ============================================================================

/// Timer engine that manages the Pomodoro timer state and events.
pub struct TimerEngine {
    /// Current timer state
    state: TimerState,
    /// Event sender channel
    event_tx: mpsc::UnboundedSender<TimerEvent>,
}

impl TimerEngine {
    /// Creates a new TimerEngine with the given configuration and event channel.
    pub fn new(config: PomodoroConfig, event_tx: mpsc::UnboundedSender<TimerEvent>) -> Self {
        Self {
            state: TimerState::new(config),
            event_tx,
        }
    }

    /// Runs the timer loop.
    ///
    /// This method runs an infinite loop that ticks every second.
    /// It should be spawned as a separate tokio task.
    pub async fn run(&mut self) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            if !self.state.is_running() {
                continue;
            }

            let completed = self.state.tick();

            // Send tick event
            self.event_tx
                .send(TimerEvent::Tick {
                    remaining_seconds: self.state.remaining_seconds,
                })
                .context("Failed to send tick event")?;

            if completed {
                self.handle_timer_complete()?;
            }
        }
    }

    /// Handles timer completion (phase transitions).
    fn handle_timer_complete(&mut self) -> Result<()> {
        match self.state.phase {
            TimerPhase::Working => {
                // Work completed - increment pomodoro count
                self.state.increment_pomodoro_count();

                self.event_tx
                    .send(TimerEvent::WorkCompleted {
                        pomodoro_count: self.state.pomodoro_count,
                        task_name: self.state.task_name.clone(),
                    })
                    .context("Failed to send work completed event")?;

                // Start break
                self.state.start_breaking();
                let is_long_break = self.state.phase == TimerPhase::LongBreaking;

                self.event_tx
                    .send(TimerEvent::BreakStarted { is_long_break })
                    .context("Failed to send break started event")?;
            }
            TimerPhase::Breaking | TimerPhase::LongBreaking => {
                let is_long_break = self.state.phase == TimerPhase::LongBreaking;

                self.event_tx
                    .send(TimerEvent::BreakCompleted { is_long_break })
                    .context("Failed to send break completed event")?;

                // Auto-cycle or stop
                if self.state.config.auto_cycle {
                    self.state.start_working(self.state.task_name.clone());

                    self.event_tx
                        .send(TimerEvent::WorkStarted {
                            task_name: self.state.task_name.clone(),
                        })
                        .context("Failed to send work started event")?;
                } else {
                    self.state.stop();
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Starts a new work session.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer is already running.
    pub fn start(&mut self, task_name: Option<String>) -> Result<()> {
        if self.state.is_running() {
            anyhow::bail!("タイマーは既に実行中です");
        }

        self.state.start_working(task_name.clone());

        self.event_tx
            .send(TimerEvent::WorkStarted { task_name })
            .context("Failed to send work started event")?;

        Ok(())
    }

    /// Pauses the timer.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer is not running.
    pub fn pause(&mut self) -> Result<()> {
        if !self.state.is_running() {
            anyhow::bail!("タイマーは実行されていません");
        }

        self.state.pause();

        self.event_tx
            .send(TimerEvent::Paused)
            .context("Failed to send paused event")?;

        Ok(())
    }

    /// Resumes a paused timer.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer is not paused.
    pub fn resume(&mut self) -> Result<()> {
        if !self.state.is_paused() {
            anyhow::bail!("タイマーは一時停止していません");
        }

        self.state.resume();

        self.event_tx
            .send(TimerEvent::Resumed)
            .context("Failed to send resumed event")?;

        Ok(())
    }

    /// Stops the timer.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer is not running or paused.
    pub fn stop(&mut self) -> Result<()> {
        if !self.state.is_running() && !self.state.is_paused() {
            anyhow::bail!("タイマーは実行されていません");
        }

        self.state.stop();

        self.event_tx
            .send(TimerEvent::Stopped)
            .context("Failed to send stopped event")?;

        Ok(())
    }

    /// Returns a reference to the current timer state.
    pub fn get_state(&self) -> &TimerState {
        &self.state
    }

    /// Returns a mutable reference to the timer state (for testing).
    #[cfg(test)]
    pub fn get_state_mut(&mut self) -> &mut TimerState {
        &mut self.state
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // TimerEvent Tests
    // ------------------------------------------------------------------------

    mod timer_event_tests {
        use super::*;

        #[test]
        fn test_work_started_event() {
            let event = TimerEvent::WorkStarted {
                task_name: Some("Test Task".to_string()),
            };
            assert_eq!(
                event,
                TimerEvent::WorkStarted {
                    task_name: Some("Test Task".to_string())
                }
            );
        }

        #[test]
        fn test_work_started_event_no_task() {
            let event = TimerEvent::WorkStarted { task_name: None };
            assert_eq!(event, TimerEvent::WorkStarted { task_name: None });
        }

        #[test]
        fn test_work_completed_event() {
            let event = TimerEvent::WorkCompleted {
                pomodoro_count: 5,
                task_name: Some("Coding".to_string()),
            };
            assert_eq!(
                event,
                TimerEvent::WorkCompleted {
                    pomodoro_count: 5,
                    task_name: Some("Coding".to_string())
                }
            );
        }

        #[test]
        fn test_break_started_event_short() {
            let event = TimerEvent::BreakStarted {
                is_long_break: false,
            };
            assert_eq!(
                event,
                TimerEvent::BreakStarted {
                    is_long_break: false
                }
            );
        }

        #[test]
        fn test_break_started_event_long() {
            let event = TimerEvent::BreakStarted {
                is_long_break: true,
            };
            assert_eq!(
                event,
                TimerEvent::BreakStarted {
                    is_long_break: true
                }
            );
        }

        #[test]
        fn test_break_completed_event() {
            let event = TimerEvent::BreakCompleted {
                is_long_break: true,
            };
            assert_eq!(
                event,
                TimerEvent::BreakCompleted {
                    is_long_break: true
                }
            );
        }

        #[test]
        fn test_paused_event() {
            let event = TimerEvent::Paused;
            assert_eq!(event, TimerEvent::Paused);
        }

        #[test]
        fn test_resumed_event() {
            let event = TimerEvent::Resumed;
            assert_eq!(event, TimerEvent::Resumed);
        }

        #[test]
        fn test_stopped_event() {
            let event = TimerEvent::Stopped;
            assert_eq!(event, TimerEvent::Stopped);
        }

        #[test]
        fn test_tick_event() {
            let event = TimerEvent::Tick {
                remaining_seconds: 1500,
            };
            assert_eq!(
                event,
                TimerEvent::Tick {
                    remaining_seconds: 1500
                }
            );
        }

        #[test]
        fn test_event_clone() {
            let event = TimerEvent::WorkStarted {
                task_name: Some("Test".to_string()),
            };
            let cloned = event.clone();
            assert_eq!(event, cloned);
        }

        #[test]
        fn test_event_debug() {
            let event = TimerEvent::Paused;
            let debug_str = format!("{:?}", event);
            assert_eq!(debug_str, "Paused");
        }
    }

    // ------------------------------------------------------------------------
    // TimerEngine Tests
    // ------------------------------------------------------------------------

    mod timer_engine_tests {
        use super::*;

        fn create_engine() -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
            let (tx, rx) = mpsc::unbounded_channel();
            let config = PomodoroConfig::default();
            let engine = TimerEngine::new(config, tx);
            (engine, rx)
        }

        fn create_engine_with_config(
            config: PomodoroConfig,
        ) -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
            let (tx, rx) = mpsc::unbounded_channel();
            let engine = TimerEngine::new(config, tx);
            (engine, rx)
        }

        #[test]
        fn test_new_engine() {
            let (engine, _rx) = create_engine();
            let state = engine.get_state();

            assert_eq!(state.phase, TimerPhase::Stopped);
            assert_eq!(state.remaining_seconds, 0);
            assert_eq!(state.pomodoro_count, 0);
        }

        #[test]
        fn test_start() {
            let (mut engine, mut rx) = create_engine();

            engine.start(Some("Test Task".to_string())).unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.remaining_seconds, 25 * 60);
            assert_eq!(state.task_name, Some("Test Task".to_string()));

            // Check event was sent
            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::WorkStarted {
                    task_name: Some("Test Task".to_string())
                }
            );
        }

        #[test]
        fn test_start_no_task() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.task_name, None);

            let event = rx.try_recv().unwrap();
            assert_eq!(event, TimerEvent::WorkStarted { task_name: None });
        }

        #[test]
        fn test_start_already_running() {
            let (mut engine, _rx) = create_engine();

            engine.start(None).unwrap();
            let result = engine.start(None);

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("既に実行中"));
        }

        #[test]
        fn test_pause() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted

            engine.pause().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Paused);

            let event = rx.try_recv().unwrap();
            assert_eq!(event, TimerEvent::Paused);
        }

        #[test]
        fn test_pause_not_running() {
            let (mut engine, _rx) = create_engine();

            let result = engine.pause();

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("実行されていません"));
        }

        #[test]
        fn test_resume() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted
            engine.pause().unwrap();
            let _ = rx.try_recv(); // consume Paused

            engine.resume().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Working);

            let event = rx.try_recv().unwrap();
            assert_eq!(event, TimerEvent::Resumed);
        }

        #[test]
        fn test_resume_not_paused() {
            let (mut engine, _rx) = create_engine();

            let result = engine.resume();

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("一時停止していません"));
        }

        #[test]
        fn test_stop_from_working() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted

            engine.stop().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Stopped);
            assert_eq!(state.remaining_seconds, 0);

            let event = rx.try_recv().unwrap();
            assert_eq!(event, TimerEvent::Stopped);
        }

        #[test]
        fn test_stop_from_paused() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv();
            engine.pause().unwrap();
            let _ = rx.try_recv();

            engine.stop().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Stopped);

            let event = rx.try_recv().unwrap();
            assert_eq!(event, TimerEvent::Stopped);
        }

        #[test]
        fn test_stop_not_running() {
            let (mut engine, _rx) = create_engine();

            let result = engine.stop();

            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("実行されていません"));
        }

        #[test]
        fn test_handle_timer_complete_work_to_break() {
            let (mut engine, mut rx) = create_engine();

            engine.start(Some("Task".to_string())).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted

            // Manually set remaining_seconds to 0 and call tick to trigger completion
            engine.get_state_mut().remaining_seconds = 1;
            let completed = engine.get_state_mut().tick();
            assert!(completed);

            engine.handle_timer_complete().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Breaking);
            assert_eq!(state.pomodoro_count, 1);

            // Check events
            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::WorkCompleted {
                    pomodoro_count: 1,
                    task_name: Some("Task".to_string())
                }
            );

            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::BreakStarted {
                    is_long_break: false
                }
            );
        }

        #[test]
        fn test_handle_timer_complete_long_break_after_4_pomodoros() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv();

            // Set pomodoro count to 3 (will become 4 after work completion)
            engine.get_state_mut().pomodoro_count = 3;
            engine.get_state_mut().remaining_seconds = 0;

            engine.handle_timer_complete().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::LongBreaking);
            assert_eq!(state.pomodoro_count, 4);

            // Check events
            let _ = rx.try_recv(); // WorkCompleted
            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::BreakStarted {
                    is_long_break: true
                }
            );
        }

        #[test]
        fn test_handle_timer_complete_break_to_stop_no_auto_cycle() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv();

            // Complete work
            engine.get_state_mut().remaining_seconds = 0;
            engine.handle_timer_complete().unwrap();
            let _ = rx.try_recv(); // WorkCompleted
            let _ = rx.try_recv(); // BreakStarted

            // Complete break
            engine.get_state_mut().remaining_seconds = 0;
            engine.handle_timer_complete().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Stopped);

            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::BreakCompleted {
                    is_long_break: false
                }
            );
        }

        #[test]
        fn test_handle_timer_complete_auto_cycle() {
            let config = PomodoroConfig {
                auto_cycle: true,
                ..PomodoroConfig::default()
            };
            let (mut engine, mut rx) = create_engine_with_config(config);

            engine.start(Some("Auto Task".to_string())).unwrap();
            let _ = rx.try_recv();

            // Complete work
            engine.get_state_mut().remaining_seconds = 0;
            engine.handle_timer_complete().unwrap();
            let _ = rx.try_recv(); // WorkCompleted
            let _ = rx.try_recv(); // BreakStarted

            // Complete break - should auto-start work
            engine.get_state_mut().remaining_seconds = 0;
            engine.handle_timer_complete().unwrap();

            let state = engine.get_state();
            assert_eq!(state.phase, TimerPhase::Working);
            assert_eq!(state.task_name, Some("Auto Task".to_string()));

            // Check events
            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::BreakCompleted {
                    is_long_break: false
                }
            );

            let event = rx.try_recv().unwrap();
            assert_eq!(
                event,
                TimerEvent::WorkStarted {
                    task_name: Some("Auto Task".to_string())
                }
            );
        }

        #[test]
        fn test_get_state() {
            let config = PomodoroConfig {
                work_minutes: 30,
                break_minutes: 10,
                ..PomodoroConfig::default()
            };
            let (engine, _rx) = create_engine_with_config(config);

            let state = engine.get_state();
            assert_eq!(state.config.work_minutes, 30);
            assert_eq!(state.config.break_minutes, 10);
        }

        #[test]
        fn test_pause_preserves_remaining_time() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv();

            // Simulate some ticks
            engine.get_state_mut().remaining_seconds = 1000;

            engine.pause().unwrap();
            let _ = rx.try_recv();

            let state = engine.get_state();
            assert_eq!(state.remaining_seconds, 1000);
        }

        #[test]
        fn test_resume_preserves_remaining_time() {
            let (mut engine, mut rx) = create_engine();

            engine.start(None).unwrap();
            let _ = rx.try_recv();

            engine.get_state_mut().remaining_seconds = 500;

            engine.pause().unwrap();
            let _ = rx.try_recv();

            engine.resume().unwrap();
            let _ = rx.try_recv();

            let state = engine.get_state();
            assert_eq!(state.remaining_seconds, 500);
        }

        #[test]
        fn test_pomodoro_count_increments_on_work_complete() {
            let (mut engine, mut rx) = create_engine();

            // First pomodoro
            engine.start(None).unwrap();
            let _ = rx.try_recv();
            engine.get_state_mut().remaining_seconds = 0;
            engine.handle_timer_complete().unwrap();

            assert_eq!(engine.get_state().pomodoro_count, 1);

            // Complete break and start second pomodoro
            engine.get_state_mut().phase = TimerPhase::Working;
            engine.get_state_mut().remaining_seconds = 0;

            // Clear events
            while rx.try_recv().is_ok() {}

            engine.handle_timer_complete().unwrap();
            assert_eq!(engine.get_state().pomodoro_count, 2);
        }

        #[test]
        fn test_long_break_at_multiples_of_4() {
            let (mut engine, _rx) = create_engine();

            engine.start(None).unwrap();

            // Test at 4, 8, 12 pomodoros
            for count in [4, 8, 12] {
                engine.get_state_mut().pomodoro_count = count - 1;
                engine.get_state_mut().phase = TimerPhase::Working;
                engine.get_state_mut().remaining_seconds = 0;

                engine.handle_timer_complete().unwrap();

                assert_eq!(
                    engine.get_state().phase,
                    TimerPhase::LongBreaking,
                    "Expected LongBreaking at pomodoro count {}",
                    count
                );
            }
        }

        #[test]
        fn test_short_break_at_non_multiples_of_4() {
            let (mut engine, _rx) = create_engine();

            engine.start(None).unwrap();

            // Test at 1, 2, 3, 5, 6, 7 pomodoros
            for count in [1, 2, 3, 5, 6, 7] {
                engine.get_state_mut().pomodoro_count = count - 1;
                engine.get_state_mut().phase = TimerPhase::Working;
                engine.get_state_mut().remaining_seconds = 0;

                engine.handle_timer_complete().unwrap();

                assert_eq!(
                    engine.get_state().phase,
                    TimerPhase::Breaking,
                    "Expected Breaking at pomodoro count {}",
                    count
                );
            }
        }
    }

    // ------------------------------------------------------------------------
    // Integration Tests with Tokio Runtime
    // ------------------------------------------------------------------------

    mod integration_tests {
        use super::*;
        use tokio::time::{timeout, Duration};

        #[tokio::test]
        async fn test_engine_run_tick_event() {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let config = PomodoroConfig::default();
            let mut engine = TimerEngine::new(config, tx);

            // Start the timer
            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted

            // Run the engine in a separate task
            let handle = tokio::spawn(async move {
                engine.run().await
            });

            // Wait for at least one tick event
            let result = timeout(Duration::from_secs(2), async {
                loop {
                    if let Ok(event) = rx.try_recv() {
                        if matches!(event, TimerEvent::Tick { .. }) {
                            return event;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await;

            // Cancel the running task
            handle.abort();

            assert!(result.is_ok(), "Should receive at least one tick event");
            let event = result.unwrap();
            assert!(matches!(event, TimerEvent::Tick { .. }));
        }

        #[tokio::test]
        async fn test_engine_run_skips_when_not_running() {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let config = PomodoroConfig::default();
            let engine = TimerEngine::new(config, tx);

            // Don't start the timer

            // Run the engine in a separate task
            let handle = tokio::spawn(async move {
                let mut engine = engine;
                engine.run().await
            });

            // Wait briefly - no events should be received
            tokio::time::sleep(Duration::from_millis(1500)).await;

            // Cancel the running task
            handle.abort();

            // Should not have received any tick events
            let event = rx.try_recv();
            assert!(
                event.is_err(),
                "Should not receive events when timer is not running"
            );
        }

        #[tokio::test]
        async fn test_engine_run_paused_no_ticks() {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let config = PomodoroConfig::default();
            let mut engine = TimerEngine::new(config, tx);

            // Start and immediately pause
            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted
            engine.pause().unwrap();
            let _ = rx.try_recv(); // consume Paused

            // Run the engine in a separate task
            let handle = tokio::spawn(async move {
                engine.run().await
            });

            // Wait briefly
            tokio::time::sleep(Duration::from_millis(1500)).await;

            // Cancel the running task
            handle.abort();

            // Should not have received any tick events (timer is paused)
            let event = rx.try_recv();
            assert!(
                event.is_err(),
                "Should not receive tick events when paused. Got: {:?}",
                event
            );
        }

        #[tokio::test]
        async fn test_timer_precision() {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let config = PomodoroConfig::default();
            let mut engine = TimerEngine::new(config, tx);

            engine.start(None).unwrap();
            let _ = rx.try_recv(); // consume WorkStarted

            let initial = engine.get_state().remaining_seconds;

            // Run the engine in a separate task
            let handle = tokio::spawn(async move {
                engine.run().await
            });

            // Wait for approximately 3 seconds
            tokio::time::sleep(Duration::from_millis(3100)).await;

            // Cancel the running task
            handle.abort();

            // Count tick events
            let mut tick_count = 0;
            while let Ok(event) = rx.try_recv() {
                if matches!(event, TimerEvent::Tick { .. }) {
                    tick_count += 1;
                }
            }

            // Should have received approximately 3 tick events (±1 for timing variance)
            assert!(
                tick_count >= 2 && tick_count <= 4,
                "Expected ~3 ticks, got {}",
                tick_count
            );
        }
    }
}
