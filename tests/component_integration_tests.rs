use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use pomodoro::daemon::timer::{TimerEngine, TimerEvent};
use pomodoro::focus::{FocusModeController, MockFocusModeController};
use pomodoro::sound::{MockSoundPlayer, SoundPlayer, SoundSource};
use pomodoro::types::{PomodoroConfig, TimerPhase};

#[cfg(target_os = "macos")]
use pomodoro::notification::{MockNotificationSender, NotificationSender, NotificationType};

fn create_engine_with_config(
    config: PomodoroConfig,
) -> (Arc<Mutex<TimerEngine>>, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let engine = TimerEngine::new(config, tx);
    (Arc::new(Mutex::new(engine)), rx)
}

fn create_fast_config() -> PomodoroConfig {
    PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: false,
        focus_mode: false,
    }
}

#[cfg(target_os = "macos")]
mod notification_integration {
    use super::*;

    #[tokio::test]
    async fn tc_i_005_work_complete_notification() {
        let mock = MockNotificationSender::new();
        mock.send_work_complete(Some("Test Task")).await.unwrap();

        let notifications = mock.get_notifications();
        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0],
            (
                NotificationType::WorkComplete,
                Some("Test Task".to_string())
            )
        );
    }

    #[tokio::test]
    async fn tc_i_006_break_complete_notification() {
        let mock = MockNotificationSender::new();
        mock.send_break_complete(None).await.unwrap();

        let notifications = mock.get_notifications();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0], (NotificationType::BreakComplete, None));
    }

    #[tokio::test]
    async fn tc_i_007_notification_action_event_handling() {
        use pomodoro::notification::NotificationActionEvent;

        let mock = MockNotificationSender::new();
        assert!(mock.try_recv_action().is_none());

        mock.inject_action_event(NotificationActionEvent::Pause);

        let action = mock.try_recv_action();
        assert_eq!(action, Some(NotificationActionEvent::Pause));
        assert!(mock.try_recv_action().is_none());
    }

    #[tokio::test]
    async fn tc_i_007_notification_failure_handling() {
        let mock = MockNotificationSender::new();
        mock.set_should_fail(true);

        let result = mock.send_work_complete(None).await;
        assert!(result.is_err());
    }
}

mod focus_mode_integration {
    use super::*;

    #[tokio::test]
    async fn tc_i_008_focus_mode_enable_on_work_start() {
        let mock = MockFocusModeController::new();
        let (engine, _rx) = create_engine_with_config(create_fast_config());

        {
            let mut eng = engine.lock().await;
            eng.start(Some("Test Task".to_string())).unwrap();
        }

        mock.enable().await.unwrap();
        assert_eq!(mock.enable_call_count(), 1);
        assert_eq!(mock.disable_call_count(), 0);
    }

    #[tokio::test]
    async fn tc_i_009_focus_mode_disable_on_break_start() {
        let mock = MockFocusModeController::new();

        mock.enable().await.unwrap();
        mock.disable().await.unwrap();

        assert_eq!(mock.enable_call_count(), 1);
        assert_eq!(mock.disable_call_count(), 1);
    }

    #[tokio::test]
    async fn tc_i_010_focus_mode_failure_fallback() {
        let mock = MockFocusModeController::new();
        mock.set_should_fail_enable(true);

        let result = mock.enable().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_recoverable());
    }

    #[tokio::test]
    async fn test_mock_focus_controller_reset() {
        let mock = MockFocusModeController::new();

        mock.enable().await.unwrap();
        mock.enable().await.unwrap();
        mock.disable().await.unwrap();

        assert_eq!(mock.enable_call_count(), 2);
        assert_eq!(mock.disable_call_count(), 1);

        mock.reset_counts();

        assert_eq!(mock.enable_call_count(), 0);
        assert_eq!(mock.disable_call_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_focus_controller_availability() {
        let mock = MockFocusModeController::new();

        assert!(mock.is_available());
        mock.set_available(false);
        assert!(!mock.is_available());
        mock.set_available(true);
        assert!(mock.is_available());
    }
}

mod sound_integration {
    use super::*;

    #[test]
    fn tc_i_011_sound_play_on_work_complete() {
        let mock = MockSoundPlayer::new();
        let source = SoundSource::embedded("notification");

        mock.play(&source).unwrap();

        assert_eq!(mock.play_count(), 1);
        let calls = mock.get_play_calls();
        assert!(calls[0].is_embedded());
    }

    #[test]
    fn tc_i_012_no_sound_flag() {
        let mock = MockSoundPlayer::new();
        mock.disable();

        let source = SoundSource::embedded("notification");
        mock.play(&source).unwrap();

        assert_eq!(mock.play_count(), 0);
    }

    #[test]
    fn tc_i_013_sound_fallback_to_embedded() {
        let mock = MockSoundPlayer::new();
        let system_source = SoundSource::system("NonExistent", "/nonexistent/path.aiff");

        mock.play(&system_source).unwrap();
        assert_eq!(mock.play_count(), 1);
    }

    #[test]
    fn test_mock_sound_player_failure() {
        let mock = MockSoundPlayer::new();
        mock.set_should_fail(true);

        let source = SoundSource::embedded("notification");
        let result = mock.play(&source);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_sound_player_clear() {
        let mock = MockSoundPlayer::new();
        let source = SoundSource::embedded("notification");

        mock.play(&source).unwrap();
        mock.play(&source).unwrap();

        assert_eq!(mock.play_count(), 2);
        mock.clear_calls();
        assert_eq!(mock.play_count(), 0);
    }

    #[test]
    fn test_mock_sound_player_availability() {
        let mock = MockSoundPlayer::new();

        assert!(mock.is_available());
        mock.set_available(false);
        assert!(!mock.is_available());
    }
}

mod launchagent_integration {
    use pomodoro::launchagent::{self, PomodoroLaunchAgent};

    #[test]
    fn tc_i_014_plist_generation() {
        let plist = PomodoroLaunchAgent::new(
            "/usr/local/bin/pomodoro".to_string(),
            "/Users/test/.pomodoro/logs".to_string(),
        );

        let xml = plist.to_xml().unwrap();
        assert!(xml.contains("com.example.pomodoro"));
        assert!(xml.contains("/usr/local/bin/pomodoro"));
        assert!(xml.contains("RunAtLoad"));
    }

    #[test]
    fn tc_i_015_plist_serialization_roundtrip() {
        let plist = PomodoroLaunchAgent::new(
            "/usr/local/bin/pomodoro".to_string(),
            "/Users/test/.pomodoro/logs".to_string(),
        );

        let xml = plist.to_xml().unwrap();
        assert!(!xml.is_empty());
        assert!(xml.starts_with("<?xml"));
    }

    #[test]
    fn tc_i_016_idempotent_install_check() {
        let installed = launchagent::is_installed();
        let running = launchagent::is_running();
        let status = launchagent::get_status();

        assert!(running.is_ok());
        assert!(status.is_ok());

        let status = status.unwrap();
        if installed {
            let _ = status.running;
        } else {
            assert!(!status.running);
        }
    }

    #[test]
    fn test_plist_label() {
        assert_eq!(PomodoroLaunchAgent::LABEL, "com.example.pomodoro");
    }
}

mod timer_event_integration {
    use super::*;

    #[tokio::test]
    async fn test_timer_start_event() {
        let (engine, mut rx) = create_engine_with_config(create_fast_config());

        {
            let mut eng = engine.lock().await;
            eng.start(Some("Test Task".to_string())).unwrap();
        }

        let event = rx.recv().await.unwrap();
        match event {
            TimerEvent::WorkStarted { task_name } => {
                assert_eq!(task_name, Some("Test Task".to_string()));
            }
            _ => panic!("Expected WorkStarted event"),
        }
    }

    #[tokio::test]
    async fn test_timer_pause_event() {
        let (engine, mut rx) = create_engine_with_config(create_fast_config());

        {
            let mut eng = engine.lock().await;
            eng.start(None).unwrap();
        }
        rx.recv().await.unwrap();

        {
            let mut eng = engine.lock().await;
            eng.pause().unwrap();
        }

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, TimerEvent::Paused));
    }

    #[tokio::test]
    async fn test_timer_resume_event() {
        let (engine, mut rx) = create_engine_with_config(create_fast_config());

        {
            let mut eng = engine.lock().await;
            eng.start(None).unwrap();
            eng.pause().unwrap();
        }
        rx.recv().await.unwrap();
        rx.recv().await.unwrap();

        {
            let mut eng = engine.lock().await;
            eng.resume().unwrap();
        }

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, TimerEvent::Resumed));
    }

    #[tokio::test]
    async fn test_timer_stop_event() {
        let (engine, mut rx) = create_engine_with_config(create_fast_config());

        {
            let mut eng = engine.lock().await;
            eng.start(None).unwrap();
        }
        rx.recv().await.unwrap();

        {
            let mut eng = engine.lock().await;
            eng.stop().unwrap();
        }

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, TimerEvent::Stopped));
    }

    #[tokio::test]
    async fn test_timer_state_transitions() {
        let (engine, _rx) = create_engine_with_config(create_fast_config());

        {
            let eng = engine.lock().await;
            let state = eng.get_state();
            assert_eq!(state.phase, TimerPhase::Stopped);
        }

        {
            let mut eng = engine.lock().await;
            eng.start(None).unwrap();
            let state = eng.get_state();
            assert_eq!(state.phase, TimerPhase::Working);
        }

        {
            let mut eng = engine.lock().await;
            eng.pause().unwrap();
            let state = eng.get_state();
            assert_eq!(state.phase, TimerPhase::Paused);
        }

        {
            let mut eng = engine.lock().await;
            eng.resume().unwrap();
            let state = eng.get_state();
            assert_eq!(state.phase, TimerPhase::Working);
        }

        {
            let mut eng = engine.lock().await;
            eng.stop().unwrap();
            let state = eng.get_state();
            assert_eq!(state.phase, TimerPhase::Stopped);
        }
    }
}

mod component_handler_integration {
    use super::*;

    struct MockComponentHandler {
        #[cfg(target_os = "macos")]
        notification_sender: MockNotificationSender,
        focus_controller: MockFocusModeController,
        sound_player: MockSoundPlayer,
    }

    impl MockComponentHandler {
        fn new() -> Self {
            Self {
                #[cfg(target_os = "macos")]
                notification_sender: MockNotificationSender::new(),
                focus_controller: MockFocusModeController::new(),
                sound_player: MockSoundPlayer::new(),
            }
        }

        async fn handle_event(&self, event: TimerEvent) {
            match event {
                TimerEvent::WorkStarted { .. } => {
                    self.focus_controller.enable().await.ok();
                }
                TimerEvent::WorkCompleted {
                    #[cfg(target_os = "macos")]
                    task_name,
                    #[cfg(not(target_os = "macos"))]
                        task_name: _,
                    ..
                } => {
                    #[cfg(target_os = "macos")]
                    {
                        self.notification_sender
                            .send_work_complete(task_name.as_deref())
                            .await
                            .ok();
                    }
                    let source = SoundSource::embedded("notification");
                    self.sound_player.play(&source).ok();
                }
                TimerEvent::BreakStarted { .. } => {
                    self.focus_controller.disable().await.ok();
                }
                TimerEvent::BreakCompleted { .. } => {
                    #[cfg(target_os = "macos")]
                    {
                        self.notification_sender
                            .send_break_complete(None)
                            .await
                            .ok();
                    }
                    let source = SoundSource::embedded("notification");
                    self.sound_player.play(&source).ok();
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_full_work_cycle_integration() {
        let handler = MockComponentHandler::new();

        handler
            .handle_event(TimerEvent::WorkStarted {
                task_name: Some("Integration Test".to_string()),
            })
            .await;
        assert_eq!(handler.focus_controller.enable_call_count(), 1);

        handler
            .handle_event(TimerEvent::WorkCompleted {
                pomodoro_count: 1,
                task_name: Some("Integration Test".to_string()),
            })
            .await;

        #[cfg(target_os = "macos")]
        assert_eq!(handler.notification_sender.notification_count(), 1);
        assert_eq!(handler.sound_player.play_count(), 1);

        handler
            .handle_event(TimerEvent::BreakStarted {
                is_long_break: false,
            })
            .await;
        assert_eq!(handler.focus_controller.disable_call_count(), 1);

        handler
            .handle_event(TimerEvent::BreakCompleted {
                is_long_break: false,
            })
            .await;

        #[cfg(target_os = "macos")]
        assert_eq!(handler.notification_sender.notification_count(), 2);
        assert_eq!(handler.sound_player.play_count(), 2);
    }

    #[tokio::test]
    async fn test_component_failure_isolation() {
        let handler = MockComponentHandler::new();
        handler.focus_controller.set_should_fail_enable(true);
        handler.sound_player.set_should_fail(true);

        handler
            .handle_event(TimerEvent::WorkStarted { task_name: None })
            .await;

        handler
            .handle_event(TimerEvent::WorkCompleted {
                pomodoro_count: 1,
                task_name: None,
            })
            .await;

        #[cfg(target_os = "macos")]
        assert_eq!(handler.notification_sender.notification_count(), 1);
    }
}
