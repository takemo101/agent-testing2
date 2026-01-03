//! Integration tests for Daemon-CLI IPC communication.
//!
//! These tests verify end-to-end communication between the CLI client
//! and the Daemon IPC server, as specified in test-specification.md:
//! - TC-I-001: Timer start via IPC
//! - TC-I-002: Timer pause via IPC
//! - TC-I-003: Status query via IPC
//! - TC-I-004: Connection error handling

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

use pomodoro_timer::cli::client::IpcClient;
use pomodoro_timer::cli::commands::StartArgs;
use pomodoro_timer::daemon::ipc::{IpcServer, RequestHandler};
use pomodoro_timer::daemon::timer::{TimerEngine, TimerEvent};
use pomodoro_timer::types::PomodoroConfig;

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a temporary socket path for testing.
fn create_temp_socket_path() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("integration_test.sock");
    // Keep the directory so it's not deleted
    std::mem::forget(dir);
    path
}

/// Creates a TimerEngine with event channel.
fn create_engine() -> (Arc<Mutex<TimerEngine>>, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    let engine = TimerEngine::new(config, tx);
    (Arc::new(Mutex::new(engine)), rx)
}

/// Runs a mock daemon server that handles a single request.
async fn run_single_request_server(
    server: &IpcServer,
    handler: &RequestHandler,
) {
    let mut stream = server.accept().await.unwrap();
    let request = IpcServer::receive_request(&mut stream).await.unwrap();
    let response = handler.handle(request).await;
    IpcServer::send_response(&mut stream, &response).await.unwrap();
}

// ============================================================================
// TC-I-001: Timer Start via IPC
// ============================================================================

/// TC-I-001: タイマー開始（IPC経由）
///
/// 前提条件: Daemon起動中
/// テスト手順:
/// 1. CLIから `start` コマンド送信
/// 2. Daemonがリクエスト受信
/// 期待結果: タイマーが開始され、成功レスポンスが返る
#[tokio::test]
async fn tc_i_001_timer_start_via_ipc() {
    // Setup
    let socket_path = create_temp_socket_path();
    let server = IpcServer::new(&socket_path).unwrap();
    let (engine, _rx) = create_engine();
    let handler = RequestHandler::new(engine);

    // Start server in background
    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Act: CLI client sends start command
    let client = IpcClient::with_socket_path(socket_path);
    let args = StartArgs {
        work: 25,
        break_time: 5,
        long_break: 15,
        task: Some("Integration Test Task".to_string()),
        auto_cycle: false,
        focus_mode: false,
        no_sound: false,
    };

    let response = client.start(&args).await;

    // Assert
    assert!(response.is_ok(), "Expected successful response, got: {:?}", response);
    let response = response.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.message, "タイマーを開始しました");

    // Verify response data
    let data = response.data.expect("Response should contain data");
    assert_eq!(data.state, Some("working".to_string()));
    assert_eq!(data.remaining_seconds, Some(25 * 60));
    assert_eq!(data.task_name, Some("Integration Test Task".to_string()));

    // Cleanup
    server_handle.abort();
}

/// TC-I-001 variant: Start with custom timer settings
#[tokio::test]
async fn tc_i_001_timer_start_with_custom_settings() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);
    let args = StartArgs {
        work: 45,          // Custom work time
        break_time: 10,    // Custom break time
        long_break: 30,    // Custom long break
        task: Some("カスタム作業".to_string()),
        auto_cycle: true,
        focus_mode: false,
        no_sound: true,
    };

    let response = client.start(&args).await.unwrap();

    assert_eq!(response.status, "success");
    let data = response.data.unwrap();
    assert_eq!(data.state, Some("working".to_string()));

    server_handle.abort();
}

// ============================================================================
// TC-I-002: Timer Pause via IPC
// ============================================================================

/// TC-I-002: タイマー一時停止（IPC経由）
///
/// 前提条件: タイマー実行中
/// テスト手順:
/// 1. CLIから `pause` コマンド送信
/// 2. Daemonがリクエスト受信
/// 期待結果: タイマーが一時停止され、成功レスポンスが返る
#[tokio::test]
async fn tc_i_002_timer_pause_via_ipc() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();

    // Pre-condition: Start the timer first
    {
        let mut eng = engine.lock().await;
        eng.start(Some("Test Task".to_string())).unwrap();
        assert!(eng.get_state().is_running());
    }

    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Act: Send pause command
    let client = IpcClient::with_socket_path(socket_path);
    let response = client.pause().await;

    // Assert
    assert!(response.is_ok(), "Expected successful response, got: {:?}", response);
    let response = response.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.message, "タイマーを一時停止しました");

    let data = response.data.expect("Response should contain data");
    assert_eq!(data.state, Some("paused".to_string()));

    server_handle.abort();
}

/// TC-I-002 variant: Pause when timer is not running (error case)
#[tokio::test]
async fn tc_i_002_timer_pause_when_not_running() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);
    let result = client.pause().await;

    // Should fail because timer is not running
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("実行されていません"),
        "Expected error about timer not running, got: {}",
        error_msg
    );

    server_handle.abort();
}

// ============================================================================
// TC-I-003: Status Query via IPC
// ============================================================================

/// TC-I-003: ステータス確認（IPC経由）
///
/// 前提条件: タイマー実行中
/// テスト手順:
/// 1. CLIから `status` コマンド送信
/// 2. Daemonがリクエスト受信
/// 期待結果: 現在の状態が返る
#[tokio::test]
async fn tc_i_003_status_query_via_ipc() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();

    // Pre-condition: Start timer and verify state
    {
        let mut eng = engine.lock().await;
        eng.start(Some("Status Test".to_string())).unwrap();
    }

    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Act
    let client = IpcClient::with_socket_path(socket_path);
    let response = client.status().await;

    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.status, "success");

    let data = response.data.expect("Response should contain data");
    assert_eq!(data.state, Some("working".to_string()));
    assert_eq!(data.remaining_seconds, Some(25 * 60));
    assert_eq!(data.pomodoro_count, Some(0));
    assert_eq!(data.task_name, Some("Status Test".to_string()));

    server_handle.abort();
}

/// TC-I-003 variant: Status query when stopped
#[tokio::test]
async fn tc_i_003_status_query_when_stopped() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);
    let response = client.status().await.unwrap();

    assert_eq!(response.status, "success");
    let data = response.data.unwrap();
    assert_eq!(data.state, Some("stopped".to_string()));
    assert_eq!(data.remaining_seconds, Some(0));

    server_handle.abort();
}

// ============================================================================
// TC-I-004: Connection Error Handling
// ============================================================================

/// TC-I-004: IPC接続エラー
///
/// 前提条件: Daemon停止中
/// テスト手順:
/// 1. CLIから `start` コマンド送信
/// 期待結果: 接続エラーメッセージが表示される
#[tokio::test]
async fn tc_i_004_connection_error_when_daemon_not_running() {
    // Use a socket path that doesn't exist (no daemon)
    let socket_path = PathBuf::from("/tmp/nonexistent_pomodoro_test_socket.sock");

    // Ensure socket doesn't exist
    let _ = std::fs::remove_file(&socket_path);

    let client = IpcClient::with_socket_path(socket_path);
    let result = client.status().await;

    // Should fail with connection error
    assert!(result.is_err(), "Expected connection error when daemon not running");

    let error_msg = result.unwrap_err().to_string();
    // The error should indicate connection failure
    assert!(
        error_msg.contains("接続") || error_msg.contains("Daemon") || error_msg.contains("connect"),
        "Expected connection error message, got: {}",
        error_msg
    );
}

/// TC-I-004 variant: Timeout on slow server
#[tokio::test]
async fn tc_i_004_connection_timeout() {
    let socket_path = create_temp_socket_path();

    // Create server that never responds
    let _server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            // Accept connection but never respond
            let _stream = server.accept().await.unwrap();
            // Sleep forever
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);

    // Use timeout to prevent test from hanging
    let result = timeout(Duration::from_secs(10), client.status()).await;

    match result {
        Ok(Err(e)) => {
            // Expected: connection error or timeout
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("タイムアウト") ||
                error_msg.contains("timeout") ||
                error_msg.contains("応答がありませんでした"),
                "Expected timeout error, got: {}",
                error_msg
            );
        }
        Ok(Ok(_)) => {
            panic!("Expected error but got success");
        }
        Err(_) => {
            // Timeout elapsed - this is also acceptable
        }
    }
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

/// Full workflow test: start -> pause -> resume -> stop
#[tokio::test]
async fn test_full_workflow_integration() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = Arc::new(RequestHandler::new(engine));

    // Server that handles multiple requests
    let handler_clone = handler.clone();
    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();

            // Handle 5 requests (start, pause, resume, stop, status)
            for _ in 0..5 {
                let mut stream = server.accept().await.unwrap();
                let request = IpcServer::receive_request(&mut stream).await.unwrap();
                let response = handler_clone.handle(request).await;
                IpcServer::send_response(&mut stream, &response).await.unwrap();
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);

    // Step 1: Start
    let response = client.start(&StartArgs::default()).await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.data.as_ref().unwrap().state, Some("working".to_string()));

    // Step 2: Pause
    let response = client.pause().await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.data.as_ref().unwrap().state, Some("paused".to_string()));

    // Step 3: Resume
    let response = client.resume().await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.data.as_ref().unwrap().state, Some("working".to_string()));

    // Step 4: Stop
    let response = client.stop().await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.data.as_ref().unwrap().state, Some("stopped".to_string()));

    // Step 5: Status
    let response = client.status().await.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.data.as_ref().unwrap().state, Some("stopped".to_string()));

    server_handle.abort();
}

/// Test Japanese task names with Unicode
#[tokio::test]
async fn test_unicode_task_name() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = RequestHandler::new(engine);

    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();
            run_single_request_server(&server, &handler).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = IpcClient::with_socket_path(socket_path);
    let args = StartArgs {
        work: 25,
        break_time: 5,
        long_break: 15,
        task: Some("🍅 ポモドーロ作業 - API実装 (v2.0)".to_string()),
        auto_cycle: false,
        focus_mode: false,
        no_sound: false,
    };

    let response = client.start(&args).await.unwrap();

    assert_eq!(response.status, "success");
    let data = response.data.unwrap();
    assert_eq!(
        data.task_name,
        Some("🍅 ポモドーロ作業 - API実装 (v2.0)".to_string())
    );

    server_handle.abort();
}

/// Test concurrent clients (sequential access)
#[tokio::test]
async fn test_concurrent_clients_sequential() {
    let socket_path = create_temp_socket_path();
    let (engine, _rx) = create_engine();
    let handler = Arc::new(RequestHandler::new(engine));

    let handler_clone = handler.clone();
    let server_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = IpcServer::new(&socket_path).unwrap();

            // Handle 3 requests
            for _ in 0..3 {
                let mut stream = server.accept().await.unwrap();
                let request = IpcServer::receive_request(&mut stream).await.unwrap();
                let response = handler_clone.handle(request).await;
                IpcServer::send_response(&mut stream, &response).await.unwrap();
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Client 1: Start
    let client1 = IpcClient::with_socket_path(socket_path.clone());
    let response1 = client1.start(&StartArgs::default()).await.unwrap();
    assert_eq!(response1.status, "success");

    // Client 2: Status (should see running)
    let client2 = IpcClient::with_socket_path(socket_path.clone());
    let response2 = client2.status().await.unwrap();
    assert_eq!(response2.data.unwrap().state, Some("working".to_string()));

    // Client 3: Stop
    let client3 = IpcClient::with_socket_path(socket_path);
    let response3 = client3.stop().await.unwrap();
    assert_eq!(response3.status, "success");

    server_handle.abort();
}
