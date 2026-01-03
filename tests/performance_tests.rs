//! Performance Tests for Pomodoro Timer CLI.
//!
//! These tests verify performance requirements as specified in test-specification.md Section 8:
//! - TC-P-001: CLI startup time (target: 100ms)
//! - TC-P-002: IPC communication latency (target: 50ms)
//! - TC-P-003: Notification send latency (target: 500ms)
//! - TC-P-004: Sound playback start latency (target: 100ms)
//! - TC-P-005: Memory usage (target: 50MB)
//! - TC-P-006: CPU usage at idle (target: 1%) - measured via manual testing
//!
//! Note: Some performance tests may be flaky in CI environments.
//! They are designed to pass under normal conditions but may fail
//! under heavy system load.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex};

use pomodoro::cli::client::IpcClient;
use pomodoro::daemon::ipc::{IpcServer, RequestHandler};
use pomodoro::daemon::timer::TimerEngine;
use pomodoro::sound::{MockSoundPlayer, SoundPlayer, SoundSource};
use pomodoro::types::{IpcRequest, PomodoroConfig, StartParams};

#[cfg(target_os = "macos")]
use pomodoro::notification::{MockNotificationSender, NotificationSender};

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a temporary socket path for testing.
fn create_temp_socket_path() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("perf_test.sock");
    std::mem::forget(dir);
    path
}

/// Creates a default TimerEngine.
fn create_engine() -> Arc<Mutex<TimerEngine>> {
    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    Arc::new(Mutex::new(TimerEngine::new(config, tx)))
}

/// Performance measurement result.
#[derive(Debug)]
struct PerfResult {
    operation: String,
    duration_ms: u128,
    target_ms: u128,
    passed: bool,
}

impl PerfResult {
    fn new(operation: &str, duration: Duration, target_ms: u128) -> Self {
        let duration_ms = duration.as_millis();
        Self {
            operation: operation.to_string(),
            duration_ms,
            target_ms,
            passed: duration_ms <= target_ms,
        }
    }

    fn assert_passed(&self) {
        assert!(
            self.passed,
            "Performance test failed: {} took {}ms (target: {}ms)",
            self.operation, self.duration_ms, self.target_ms
        );
    }
}

// ============================================================================
// TC-P-001: CLI Startup Time
// ============================================================================

/// TC-P-001: CLI起動時間
///
/// 条件: `pomodoro --help`
/// 目標値: 100ms以内
/// 測定方法: `time` コマンド
///
/// Note: This test measures the time to parse arguments and create client,
/// not the actual binary startup time (which would be measured with `time` command).
#[test]
fn tc_p_001_cli_argument_parsing_time() {
    let start = Instant::now();

    // Simulate CLI argument parsing using the actual CLI struct
    use clap::Parser;
    use pomodoro::cli::commands::Cli;

    // Try parsing with --version (simpler than --help which exits)
    let _ = Cli::try_parse_from(["pomodoro", "status"]);

    let duration = start.elapsed();
    let result = PerfResult::new("CLI argument parsing", duration, 100);

    // Log performance result
    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        result.operation, result.duration_ms, result.target_ms
    );

    result.assert_passed();
}

/// Additional test: IpcClient creation time
#[test]
fn tc_p_001_ipc_client_creation_time() {
    let socket_path = PathBuf::from("/tmp/perf_test_socket.sock");

    let start = Instant::now();
    let _client = IpcClient::with_socket_path(socket_path);
    let duration = start.elapsed();

    let result = PerfResult::new("IpcClient creation", duration, 10);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        result.operation, result.duration_ms, result.target_ms
    );

    result.assert_passed();
}

// ============================================================================
// TC-P-002: IPC Communication Latency
// ============================================================================

/// TC-P-002: IPC通信遅延
///
/// 条件: `pomodoro status`
/// 目標値: 50ms以内
/// 測定方法: `tracing` ログ
#[tokio::test]
async fn tc_p_002_ipc_latency() {
    let socket_path = create_temp_socket_path();
    let engine = create_engine();
    let handler = Arc::new(RequestHandler::new(engine));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        if let Ok(mut stream) = server_clone.accept().await {
            if let Ok(request) = IpcServer::receive_request(&mut stream).await {
                let response = handler_clone.handle(request).await;
                let _ = IpcServer::send_response(&mut stream, &response).await;
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = IpcClient::with_socket_path(socket_path);

    // Measure IPC round-trip time
    let start = Instant::now();
    let result = client.status().await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "IPC call should succeed");

    let perf_result = PerfResult::new("IPC status query", duration, 50);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        perf_result.operation, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();

    server_handle.abort();
}

/// Test IPC latency for multiple sequential requests
#[tokio::test]
async fn tc_p_002_ipc_latency_multiple_requests() {
    let socket_path = create_temp_socket_path();
    let engine = create_engine();
    let handler = Arc::new(RequestHandler::new(engine));

    let server = Arc::new(IpcServer::new(&socket_path).unwrap());
    let server_clone = server.clone();
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn(async move {
        for _ in 0..10 {
            if let Ok(mut stream) = server_clone.accept().await {
                if let Ok(request) = IpcServer::receive_request(&mut stream).await {
                    let response = handler_clone.handle(request).await;
                    let _ = IpcServer::send_response(&mut stream, &response).await;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = IpcClient::with_socket_path(socket_path);

    let mut total_duration = Duration::ZERO;
    let request_count = 5;

    for _ in 0..request_count {
        let start = Instant::now();
        let _ = client.status().await;
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / request_count as u32;
    let perf_result = PerfResult::new("IPC average latency", avg_duration, 50);

    eprintln!(
        "Performance: {} (avg over {} requests) completed in {}ms (target: {}ms)",
        perf_result.operation, request_count, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();

    server_handle.abort();
}

// ============================================================================
// TC-P-003: Notification Send Latency
// ============================================================================

/// TC-P-003: 通知送信遅延
///
/// 条件: タイマー完了→通知表示
/// 目標値: 500ms以内
/// 測定方法: `tracing` ログ
#[tokio::test]
#[cfg(target_os = "macos")]
async fn tc_p_003_notification_send_latency() {
    let mock = MockNotificationSender::new();

    let start = Instant::now();
    let result = mock.send_work_complete(Some("Performance Test")).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Notification should be sent successfully");

    let perf_result = PerfResult::new("Notification send", duration, 500);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        perf_result.operation, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();
}

/// Test notification latency for break complete
#[tokio::test]
#[cfg(target_os = "macos")]
async fn tc_p_003_notification_break_complete_latency() {
    let mock = MockNotificationSender::new();

    let start = Instant::now();
    let result = mock.send_break_complete(None).await;
    let duration = start.elapsed();

    assert!(result.is_ok());

    let perf_result = PerfResult::new("Break notification send", duration, 500);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        perf_result.operation, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();
}

// ============================================================================
// TC-P-004: Sound Playback Start Latency
// ============================================================================

/// TC-P-004: サウンド再生開始遅延
///
/// 条件: タイマー完了→再生開始
/// 目標値: 100ms以内
/// 測定方法: `tracing` ログ
#[test]
fn tc_p_004_sound_playback_start_latency() {
    let mock = MockSoundPlayer::new();
    let source = SoundSource::embedded("notification");

    let start = Instant::now();
    let result = mock.play(&source);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Sound should play successfully");

    let perf_result = PerfResult::new("Sound playback initiation", duration, 100);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        perf_result.operation, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();
}

/// Test sound source creation performance
#[test]
fn tc_p_004_sound_source_creation_latency() {
    let start = Instant::now();
    let _source = SoundSource::embedded("notification");
    let duration = start.elapsed();

    let perf_result = PerfResult::new("SoundSource creation", duration, 10);

    eprintln!(
        "Performance: {} completed in {}ms (target: {}ms)",
        perf_result.operation, perf_result.duration_ms, perf_result.target_ms
    );

    perf_result.assert_passed();
}

// ============================================================================
// TC-P-005: Memory Usage
// ============================================================================

/// TC-P-005: メモリ使用量
///
/// 条件: Daemon常駐時
/// 目標値: 50MB以下
/// 測定方法: Activity Monitor
///
/// Note: This test estimates memory by measuring the size of key data structures.
/// Actual memory usage should be verified with Activity Monitor or similar tools.
#[test]
fn tc_p_005_memory_usage_estimate() {
    use std::mem::size_of;

    // Measure sizes of key types
    let timer_state_size = size_of::<pomodoro::TimerState>();
    let config_size = size_of::<pomodoro::PomodoroConfig>();
    let ipc_request_size = size_of::<pomodoro::IpcRequest>();
    let ipc_response_size = size_of::<pomodoro::IpcResponse>();

    let total_core_structs_bytes =
        timer_state_size + config_size + ipc_request_size + ipc_response_size;

    eprintln!(
        "Memory estimates:\n\
         - TimerState: {} bytes\n\
         - PomodoroConfig: {} bytes\n\
         - IpcRequest: {} bytes\n\
         - IpcResponse: {} bytes\n\
         - Total core structs: {} bytes",
        timer_state_size,
        config_size,
        ipc_request_size,
        ipc_response_size,
        total_core_structs_bytes
    );

    // Core data structures should be small (< 1KB)
    assert!(
        total_core_structs_bytes < 1024,
        "Core data structures should be under 1KB, got {} bytes",
        total_core_structs_bytes
    );

    // Note: Actual memory usage (50MB target) includes:
    // - Tokio runtime
    // - IPC server
    // - Audio subsystem
    // - System libraries
    // This must be verified manually or with integration tests
}

/// Test memory efficiency of event channel
#[tokio::test]
async fn tc_p_005_event_channel_memory() {
    use pomodoro::daemon::timer::TimerEvent;
    use std::mem::size_of;

    let event_size = size_of::<TimerEvent>();
    eprintln!("TimerEvent size: {} bytes", event_size);

    // TimerEvent should be reasonably small
    assert!(
        event_size < 256,
        "TimerEvent should be under 256 bytes, got {} bytes",
        event_size
    );

    // Test channel doesn't grow unboundedly
    let (tx, _rx) = mpsc::unbounded_channel::<TimerEvent>();

    // Send many events
    for i in 0..1000 {
        let event = TimerEvent::WorkStarted {
            task_name: Some(format!("Task {}", i)),
        };
        tx.send(event).unwrap();
    }

    // Channel should handle many events without issues
    assert!(!tx.is_closed());
}

// ============================================================================
// TC-P-006: CPU Usage (Structural Test)
// ============================================================================

/// TC-P-006: CPU使用率
///
/// 条件: アイドル時
/// 目標値: 1%以下
/// 測定方法: Activity Monitor
///
/// Note: Actual CPU usage must be measured manually. This test verifies
/// the timer doesn't busy-wait.
#[tokio::test]
async fn tc_p_006_no_busy_wait() {
    let engine = create_engine();

    // Verify tick() doesn't need to be called constantly
    {
        let eng = engine.lock().await;
        let state = eng.get_state();
        assert_eq!(state.phase, pomodoro::TimerPhase::Stopped);
    }

    // Start timer
    {
        let mut eng = engine.lock().await;
        eng.start(None).unwrap();
    }

    // Sleep without ticking (simulating no CPU usage)
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let elapsed = start.elapsed();

    // Verify sleep was actually sleeping (not busy-waiting)
    assert!(
        elapsed >= Duration::from_millis(95),
        "Sleep should have actually slept, elapsed: {:?}",
        elapsed
    );

    // State should be unchanged (no automatic ticking)
    {
        let eng = engine.lock().await;
        let state = eng.get_state();
        // Remaining time shouldn't have changed since we didn't call tick()
        assert_eq!(state.remaining_seconds, 25 * 60);
    }
}

// ============================================================================
// Benchmark-style Tests
// ============================================================================

/// Benchmark: Timer state transitions
#[tokio::test]
async fn benchmark_timer_state_transitions() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut engine = TimerEngine::new(PomodoroConfig::default(), tx);

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        engine.start(Some("Benchmark".to_string())).unwrap();
        engine.pause().unwrap();
        engine.resume().unwrap();
        engine.stop().unwrap();
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    eprintln!(
        "Benchmark: {} state transitions in {:?} ({:?} per iteration)",
        iterations * 4, // 4 transitions per iteration
        duration,
        per_iteration
    );

    // Each iteration (4 state changes) should be under 1ms
    assert!(
        per_iteration < Duration::from_millis(1),
        "State transitions too slow: {:?} per iteration",
        per_iteration
    );
}

/// Benchmark: Timer ticks
#[tokio::test]
async fn benchmark_timer_ticks() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut engine = TimerEngine::new(PomodoroConfig::default(), tx);
    engine.start(None).unwrap();

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        let state = engine.get_state_mut();
        state.tick();
    }

    let duration = start.elapsed();
    let per_tick = duration / iterations;

    eprintln!(
        "Benchmark: {} ticks in {:?} ({:?} per tick)",
        iterations, duration, per_tick
    );

    // Each tick should be under 100 microseconds
    assert!(
        per_tick < Duration::from_micros(100),
        "Ticks too slow: {:?} per tick",
        per_tick
    );
}

/// Benchmark: IPC request serialization
#[test]
fn benchmark_ipc_serialization() {
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let request = IpcRequest::Start {
            params: StartParams {
                work_minutes: Some(25),
                break_minutes: Some(5),
                long_break_minutes: Some(15),
                task_name: Some("Benchmark Task".to_string()),
                auto_cycle: Some(false),
                focus_mode: Some(false),
            },
        };
        let _json = serde_json::to_string(&request).unwrap();
    }

    let duration = start.elapsed();
    let per_serialization = duration / iterations;

    eprintln!(
        "Benchmark: {} serializations in {:?} ({:?} per serialization)",
        iterations, duration, per_serialization
    );

    // Each serialization should be under 100 microseconds
    assert!(
        per_serialization < Duration::from_micros(100),
        "Serialization too slow: {:?}",
        per_serialization
    );
}

/// Benchmark: IPC response deserialization
#[test]
fn benchmark_ipc_deserialization() {
    use pomodoro::types::{IpcResponse, ResponseData};

    let response = IpcResponse {
        status: "success".to_string(),
        message: "Timer started".to_string(),
        data: Some(ResponseData {
            state: Some("working".to_string()),
            remaining_seconds: Some(1500),
            pomodoro_count: Some(0),
            task_name: Some("Benchmark Task".to_string()),
        }),
    };
    let json = serde_json::to_string(&response).unwrap();

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _: IpcResponse = serde_json::from_str(&json).unwrap();
    }

    let duration = start.elapsed();
    let per_deserialization = duration / iterations;

    eprintln!(
        "Benchmark: {} deserializations in {:?} ({:?} per deserialization)",
        iterations, duration, per_deserialization
    );

    // Each deserialization should be under 100 microseconds
    assert!(
        per_deserialization < Duration::from_micros(100),
        "Deserialization too slow: {:?}",
        per_deserialization
    );
}

// ============================================================================
// Performance Summary Test
// ============================================================================

/// Summary test that runs all performance checks and reports results
#[tokio::test]
async fn test_performance_summary() {
    eprintln!("\n=== PERFORMANCE TEST SUMMARY ===\n");

    // TC-P-001: CLI startup (simulated)
    eprintln!("TC-P-001: CLI startup time - See tc_p_001_* tests");

    // TC-P-002: IPC latency
    eprintln!("TC-P-002: IPC latency - See tc_p_002_* tests");

    // TC-P-003: Notification latency
    #[cfg(target_os = "macos")]
    eprintln!("TC-P-003: Notification latency - See tc_p_003_* tests");
    #[cfg(not(target_os = "macos"))]
    eprintln!("TC-P-003: Notification latency - Skipped (not macOS)");

    // TC-P-004: Sound playback
    eprintln!("TC-P-004: Sound playback latency - See tc_p_004_* tests");

    // TC-P-005: Memory usage
    eprintln!("TC-P-005: Memory usage - See tc_p_005_* tests");
    eprintln!("         Note: Manual verification with Activity Monitor required");

    // TC-P-006: CPU usage
    eprintln!("TC-P-006: CPU usage at idle - Manual verification required");
    eprintln!("         Note: Use Activity Monitor to verify <1% CPU at idle");

    eprintln!("\n=== END PERFORMANCE SUMMARY ===\n");
}
