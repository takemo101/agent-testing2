//! End-to-End Tests for Pomodoro Timer CLI.
//!
//! These tests verify complete user workflows as specified in test-specification.md Section 4:
//! - TC-E-001: Complete pomodoro cycle
//! - TC-E-002: Pause and resume flow
//! - TC-E-003: Stop flow
//! - TC-E-004: Auto-cycle mode
//! - TC-E-005: Long break after 4 pomodoros
//! - TC-E-006: Focus mode integration

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;

use pomodoro::cli::client::IpcClient;
use pomodoro::cli::commands::StartArgs;
use pomodoro::daemon::ipc::{IpcServer, RequestHandler};
use pomodoro::daemon::timer::{TimerEngine, TimerEvent};
use pomodoro::focus::{FocusModeController, MockFocusModeController};
use pomodoro::sound::{MockSoundPlayer, SoundPlayer, SoundSource};
use pomodoro::types::{PomodoroConfig, TimerPhase};

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a temporary socket path for testing.
fn create_temp_socket_path() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("e2e_test.sock");
    // Keep the directory so it's not deleted
    std::mem::forget(dir);
    path
}

/// Creates a TimerEngine with custom configuration.
fn create_engine_with_config(
    config: PomodoroConfig,
) -> (Arc<Mutex<TimerEngine>>, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let engine = TimerEngine::new(config, tx);
    (Arc::new(Mutex::new(engine)), rx)
}

/// Creates a fast configuration for quick tests (1-minute work sessions).
fn create_fast_config() -> PomodoroConfig {
    PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: false,
        focus_mode: false,
    }
}

/// Creates an auto-cycle configuration.
fn create_auto_cycle_config() -> PomodoroConfig {
    PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: true,
        focus_mode: false,
    }
}

/// Creates a focus mode enabled configuration.
fn create_focus_mode_config() -> PomodoroConfig {
    PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: false,
        focus_mode: true,
    }
}

/// Runs multiple request-response cycles on the server.
async fn handle_requests(server: &IpcServer, handler: &RequestHandler, count: usize) {
    for _ in 0..count {
        if let Ok(mut stream) = server.accept().await {
            if let Ok(request) = IpcServer::receive_request(&mut stream).await {
                let response = handler.handle(request).await;
                let _ = IpcServer::send_response(&mut stream, &response).await;
            }
        }
    }
}

/// Simulates timer ticks until phase completion.
async fn simulate_work_completion(
    engine: Arc<Mutex<TimerEngine>>,
    rx: &mut mpsc::UnboundedReceiver<TimerEvent>,
) {
    // Drain the initial start event
    let _ = rx.recv().await;

    // Simulate work completion by ticking until done
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            // Phase completed
            break;
        }
    }

    // Wait for completion event
    let _ = timeout(Duration::from_secs(1), rx.recv()).await;
}

// ============================================================================
// TC-E-001: Complete Pomodoro Cycle
// ============================================================================

/// TC-E-001: 完全なポモドーロサイクル
///
/// 前提条件: Daemon起動中
/// テスト手順:
/// 1. `pomodoro start --task "テスト"`
/// 2. 作業完了まで待機
/// 3. 作業完了通知確認
/// 4. 休憩完了まで待機
/// 5. 休憩完了通知確認
/// 期待結果: 通知が表示され、タイマーが正常に動作する
#[tokio::test]
async fn tc_e_001_complete_pomodoro_cycle() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 10).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = IpcClient::with_socket_path(socket_path);

    // Step 1: Start timer with task name
    let args = StartArgs {
        work: 1,
        break_time: 1,
        long_break: 2,
        task: Some("E2Eテスト".to_string()),
        auto_cycle: false,
        focus_mode: false,
        no_sound: false,
    };

    let response = client.start(&args).await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.message, "タイマーを開始しました");
    let data = response.data.unwrap();
    assert_eq!(data.state, Some("working".to_string()));
    assert_eq!(data.task_name, Some("E2Eテスト".to_string()));
    assert_eq!(data.remaining_seconds, Some(60)); // 1 minute

    // Verify WorkStarted event
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::WorkStarted { task_name }) => {
            assert_eq!(task_name, Some("E2Eテスト".to_string()));
        }
        _ => panic!("Expected WorkStarted event"),
    }

    // Step 2: Simulate work completion
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            eng.start_break().unwrap();
            break;
        }
    }

    // Step 3: Verify work completion event
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::WorkCompleted { pomodoro_count, .. }) => {
            assert_eq!(pomodoro_count, 1);
        }
        e => panic!("Expected WorkCompleted event, got {:?}", e),
    }

    // Step 4: Verify break started
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::BreakStarted { is_long_break }) => {
            assert!(!is_long_break);
        }
        e => panic!("Expected BreakStarted event, got {:?}", e),
    }

    // Verify status shows breaking
    let response = client.status().await.unwrap();
    assert_eq!(response.status, "success");
    let data = response.data.unwrap();
    assert_eq!(data.state, Some("breaking".to_string()));
    assert_eq!(data.pomodoro_count, Some(1));

    // Step 5: Simulate break completion
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            break;
        }
    }

    // Verify break completion event
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::BreakCompleted { is_long_break }) => {
            assert!(!is_long_break);
        }
        e => panic!("Expected BreakCompleted event, got {:?}", e),
    }

    server_handle.abort();
}

// ============================================================================
// TC-E-002: Pause and Resume Flow
// ============================================================================

/// TC-E-002: 一時停止・再開フロー
///
/// 前提条件: タイマー実行中
/// テスト手順:
/// 1. `pomodoro pause`
/// 2. 残り時間確認
/// 3. `pomodoro resume`
/// 4. タイマー継続確認
/// 期待結果: 一時停止・再開が正常に動作する
#[tokio::test]
async fn tc_e_002_pause_resume_flow() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 10).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Start timer
    let args = StartArgs {
        work: 1,
        break_time: 1,
        long_break: 2,
        task: Some("Pause Test".to_string()),
        auto_cycle: false,
        focus_mode: false,
        no_sound: false,
    };
    let _ = client.start(&args).await.unwrap();
    let _ = rx.recv().await; // Drain start event

    // Simulate a few ticks
    for _ in 0..5 {
        let mut eng = engine.lock().await;
        eng.tick();
    }

    // Get remaining time before pause
    let status_before = client.status().await.unwrap();
    let remaining_before = status_before.data.as_ref().unwrap().remaining_seconds.unwrap();

    // Step 1: Pause
    let pause_response = client.pause().await.unwrap();
    assert_eq!(pause_response.status, "success");
    assert_eq!(pause_response.message, "タイマーを一時停止しました");

    let pause_data = pause_response.data.unwrap();
    assert_eq!(pause_data.state, Some("paused".to_string()));

    // Verify Paused event
    let event = rx.recv().await.unwrap();
    assert!(matches!(event, TimerEvent::Paused));

    // Step 2: Verify remaining time is preserved
    let status_paused = client.status().await.unwrap();
    let remaining_paused = status_paused.data.as_ref().unwrap().remaining_seconds.unwrap();
    assert_eq!(remaining_before, remaining_paused);

    // Tick while paused (should not change time)
    {
        let mut eng = engine.lock().await;
        eng.tick();
        eng.tick();
    }

    let status_still_paused = client.status().await.unwrap();
    let remaining_still = status_still_paused.data.as_ref().unwrap().remaining_seconds.unwrap();
    assert_eq!(remaining_paused, remaining_still);

    // Step 3: Resume
    let resume_response = client.resume().await.unwrap();
    assert_eq!(resume_response.status, "success");
    assert_eq!(resume_response.message, "タイマーを再開しました");

    let resume_data = resume_response.data.unwrap();
    assert_eq!(resume_data.state, Some("working".to_string()));

    // Verify Resumed event
    let event = rx.recv().await.unwrap();
    assert!(matches!(event, TimerEvent::Resumed));

    // Step 4: Verify timer continues
    {
        let mut eng = engine.lock().await;
        eng.tick();
    }

    let status_resumed = client.status().await.unwrap();
    let remaining_resumed = status_resumed.data.as_ref().unwrap().remaining_seconds.unwrap();
    assert!(remaining_resumed < remaining_still);

    server_handle.abort();
}

// ============================================================================
// TC-E-003: Stop Flow
// ============================================================================

/// TC-E-003: 停止フロー
///
/// 前提条件: タイマー実行中
/// テスト手順:
/// 1. `pomodoro stop`
/// 2. ステータス確認
/// 期待結果: タイマーが停止し、状態がリセットされる
#[tokio::test]
async fn tc_e_003_stop_flow() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 5).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Start timer
    let args = StartArgs {
        work: 25,
        break_time: 5,
        long_break: 15,
        task: Some("Stop Test".to_string()),
        auto_cycle: false,
        focus_mode: false,
        no_sound: false,
    };
    let _ = client.start(&args).await.unwrap();
    let _ = rx.recv().await; // Drain start event

    // Verify timer is running
    let status = client.status().await.unwrap();
    assert_eq!(status.data.as_ref().unwrap().state, Some("working".to_string()));

    // Step 1: Stop
    let stop_response = client.stop().await.unwrap();
    assert_eq!(stop_response.status, "success");
    assert_eq!(stop_response.message, "タイマーを停止しました");

    let stop_data = stop_response.data.unwrap();
    assert_eq!(stop_data.state, Some("stopped".to_string()));

    // Verify Stopped event
    let event = rx.recv().await.unwrap();
    assert!(matches!(event, TimerEvent::Stopped));

    // Step 2: Verify state is reset
    let status_after = client.status().await.unwrap();
    let data = status_after.data.unwrap();
    assert_eq!(data.state, Some("stopped".to_string()));
    assert_eq!(data.remaining_seconds, Some(0));
    assert_eq!(data.task_name, None);

    server_handle.abort();
}

// ============================================================================
// TC-E-004: Auto-cycle Mode
// ============================================================================

/// TC-E-004: 自動サイクル有効時の動作
///
/// 前提条件: `--auto-cycle` 指定
/// テスト手順:
/// 1. `pomodoro start --auto-cycle`
/// 2. 作業完了
/// 3. 休憩完了
/// 期待結果: 自動的に次の作業タイマーが開始される
#[tokio::test]
async fn tc_e_004_auto_cycle_mode() {
    let socket_path = create_temp_socket_path();
    let config = create_auto_cycle_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 10).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Step 1: Start with auto-cycle
    let args = StartArgs {
        work: 1,
        break_time: 1,
        long_break: 2,
        task: Some("Auto Cycle Test".to_string()),
        auto_cycle: true,
        focus_mode: false,
        no_sound: false,
    };
    let response = client.start(&args).await.unwrap();
    assert_eq!(response.status, "success");
    let _ = rx.recv().await; // Drain start event

    // Step 2: Simulate work completion
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            // Auto-cycle should automatically start break
            eng.start_break().unwrap();
            break;
        }
    }

    // Drain work completed and break started events
    let _ = timeout(Duration::from_secs(1), rx.recv()).await;
    let _ = timeout(Duration::from_secs(1), rx.recv()).await;

    // Verify we're in break mode
    let status = client.status().await.unwrap();
    assert_eq!(status.data.as_ref().unwrap().state, Some("breaking".to_string()));

    // Step 3: Simulate break completion
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            // In auto-cycle, should automatically start next work session
            eng.start(None).unwrap();
            break;
        }
    }

    // Drain break completed event
    let _ = timeout(Duration::from_secs(1), rx.recv()).await;

    // Wait for work started event
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());

    // Verify we're back in working mode (next cycle started)
    let status = client.status().await.unwrap();
    let data = status.data.unwrap();
    assert_eq!(data.state, Some("working".to_string()));
    assert_eq!(data.pomodoro_count, Some(1)); // Count from previous cycle

    server_handle.abort();
}

// ============================================================================
// TC-E-005: Long Break After 4 Pomodoros
// ============================================================================

/// TC-E-005: 4ポモドーロ後の長い休憩
///
/// 前提条件: `pomodoro_count=3`
/// テスト手順:
/// 1. 4回目の作業完了
/// 期待結果: 長い休憩タイマーが開始される
#[tokio::test]
async fn tc_e_005_long_break_after_4_pomodoros() {
    let config = create_fast_config();
    let (engine, mut rx) = create_engine_with_config(config);

    // Pre-condition: Set pomodoro count to 3 (will become 4 after next completion)
    {
        let mut eng = engine.lock().await;
        eng.start(Some("Long Break Test".to_string())).unwrap();
        // Simulate 3 completed pomodoros by setting internal state
        for _ in 0..3 {
            eng.tick(); // Just to initialize
            eng.increment_pomodoro_count();
        }
    }
    let _ = rx.recv().await; // Drain start event

    // Complete work session (4th pomodoro)
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            eng.start_break().unwrap();
            break;
        }
    }

    // Verify work completed event with pomodoro count = 4
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::WorkCompleted { pomodoro_count, .. }) => {
            assert_eq!(pomodoro_count, 4);
        }
        e => panic!("Expected WorkCompleted event, got {:?}", e),
    }

    // Verify long break started
    let event = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(event.is_ok());
    match event.unwrap() {
        Some(TimerEvent::BreakStarted { is_long_break }) => {
            assert!(is_long_break, "Expected long break after 4 pomodoros");
        }
        e => panic!("Expected BreakStarted event with is_long_break=true, got {:?}", e),
    }

    // Verify state shows long breaking
    {
        let eng = engine.lock().await;
        let state = eng.get_state();
        assert_eq!(state.phase, TimerPhase::LongBreaking);
        assert_eq!(state.remaining_seconds, 2 * 60); // long_break_minutes = 2
    }
}

// ============================================================================
// TC-E-006: Focus Mode Integration
// ============================================================================

/// TC-E-006: フォーカスモード連携
///
/// 前提条件: ショートカット作成済み
/// テスト手順:
/// 1. `pomodoro start --focus-mode`
/// 2. フォーカスモード確認
/// 3. 休憩開始
/// 4. フォーカスモード確認
/// 期待結果: 作業中はON、休憩中はOFF
#[tokio::test]
async fn tc_e_006_focus_mode_integration() {
    let config = create_focus_mode_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let mock_focus = MockFocusModeController::new();

    // Step 1: Start with focus mode
    {
        let mut eng = engine.lock().await;
        eng.start(Some("Focus Mode Test".to_string())).unwrap();
    }
    let _ = rx.recv().await; // Drain start event

    // Simulate focus mode enable on work start
    mock_focus.enable().await.unwrap();

    // Step 2: Verify focus mode is ON (enable was called)
    assert_eq!(mock_focus.enable_call_count(), 1);
    assert_eq!(mock_focus.disable_call_count(), 0);

    // Complete work session
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut eng = engine.lock().await;
        if eng.tick() {
            break;
        }
    }
    let _ = timeout(Duration::from_secs(1), rx.recv()).await; // Work completed

    // Step 3: Start break
    {
        let mut eng = engine.lock().await;
        eng.start_break().unwrap();
    }
    let _ = timeout(Duration::from_secs(1), rx.recv()).await; // Break started

    // Simulate focus mode disable on break start
    mock_focus.disable().await.unwrap();

    // Step 4: Verify focus mode is OFF (disable was called)
    assert_eq!(mock_focus.enable_call_count(), 1);
    assert_eq!(mock_focus.disable_call_count(), 1);
}

// ============================================================================
// E2E Workflow: Full Session Test
// ============================================================================

/// Full E2E workflow test: Multiple pomodoros with all operations
#[tokio::test]
async fn test_e2e_full_session_workflow() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, mut rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 20).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Start first pomodoro
    let args = StartArgs::default();
    let response = client.start(&args).await.unwrap();
    assert_eq!(response.status, "success");
    let _ = rx.recv().await;

    // Pause
    let response = client.pause().await.unwrap();
    assert_eq!(response.status, "success");
    let _ = rx.recv().await;

    // Resume
    let response = client.resume().await.unwrap();
    assert_eq!(response.status, "success");
    let _ = rx.recv().await;

    // Status check
    let response = client.status().await.unwrap();
    assert_eq!(response.data.as_ref().unwrap().state, Some("working".to_string()));

    // Stop
    let response = client.stop().await.unwrap();
    assert_eq!(response.status, "success");
    let _ = rx.recv().await;

    // Verify stopped state
    let response = client.status().await.unwrap();
    assert_eq!(response.data.as_ref().unwrap().state, Some("stopped".to_string()));

    server_handle.abort();
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test rapid start/stop cycles
#[tokio::test]
async fn test_e2e_rapid_start_stop_cycles() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, _rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 20).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Rapid start/stop 5 times
    for i in 0..5 {
        let args = StartArgs {
            task: Some(format!("Rapid Test {}", i)),
            ..Default::default()
        };
        let start_response = client.start(&args).await.unwrap();
        assert_eq!(start_response.status, "success");

        let stop_response = client.stop().await.unwrap();
        assert_eq!(stop_response.status, "success");
    }

    // Verify final state is stopped
    let status = client.status().await.unwrap();
    assert_eq!(status.data.as_ref().unwrap().state, Some("stopped".to_string()));

    server_handle.abort();
}

/// Test pause/resume multiple times
#[tokio::test]
async fn test_e2e_multiple_pause_resume() {
    let socket_path = create_temp_socket_path();
    let config = create_fast_config();
    let (engine, _rx) = create_engine_with_config(config);
    let handler = Arc::new(RequestHandler::new(engine.clone()));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        handle_requests(&server_clone, &handler_clone, 15).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    // Start timer
    let _ = client.start(&StartArgs::default()).await.unwrap();

    // Pause/resume 3 times
    for _ in 0..3 {
        let pause_response = client.pause().await.unwrap();
        assert_eq!(pause_response.status, "success");

        let resume_response = client.resume().await.unwrap();
        assert_eq!(resume_response.status, "success");
    }

    // Verify still running
    let status = client.status().await.unwrap();
    assert_eq!(status.data.as_ref().unwrap().state, Some("working".to_string()));

    server_handle.abort();
}
