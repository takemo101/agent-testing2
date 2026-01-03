//! IPC Client for communicating with the Pomodoro Timer daemon.
//!
//! This module provides:
//! - Unix Domain Socket client
//! - Request/response handling
//! - Connection retry logic
//! - Timeout handling

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::cli::commands::StartArgs;
use crate::types::{IpcRequest, IpcResponse, StartParams};

// ============================================================================
// Constants
// ============================================================================

/// Default socket path
const DEFAULT_SOCKET_PATH: &str = ".pomodoro/pomodoro.sock";

/// Connection timeout in seconds
const CONNECTION_TIMEOUT_SECS: u64 = 5;

/// Read/write timeout in seconds
const IO_TIMEOUT_SECS: u64 = 5;

/// Maximum response size in bytes (64KB)
const MAX_RESPONSE_SIZE: usize = 65536;

/// Maximum retry attempts
const MAX_RETRIES: u32 = 3;

/// Retry delay in milliseconds (base delay, multiplied by attempt number)
const RETRY_DELAY_MS: u64 = 500;

// ============================================================================
// IpcClient
// ============================================================================

/// IPC client for daemon communication.
pub struct IpcClient {
    /// Socket path
    socket_path: PathBuf,
    /// Connection timeout
    timeout: Duration,
}

impl IpcClient {
    /// Creates a new IPC client with default socket path.
    pub fn new() -> Result<Self> {
        let socket_path = Self::default_socket_path()?;
        Ok(Self {
            socket_path,
            timeout: Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        })
    }

    /// Creates a new IPC client with a custom socket path.
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            timeout: Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        }
    }

    /// Returns the default socket path.
    fn default_socket_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME環境変数が設定されていません")?;
        Ok(PathBuf::from(home).join(DEFAULT_SOCKET_PATH))
    }

    /// Returns the socket path.
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Sends a start command to the daemon.
    pub async fn start(&self, args: &StartArgs) -> Result<IpcResponse> {
        let params = StartParams {
            work_minutes: Some(args.work),
            break_minutes: Some(args.break_time),
            long_break_minutes: Some(args.long_break),
            task_name: args.task.clone(),
            auto_cycle: Some(args.auto_cycle),
            focus_mode: Some(args.focus_mode),
        };

        let request = IpcRequest::Start { params };
        self.send_request_with_retry(&request).await
    }

    /// Sends a pause command to the daemon.
    pub async fn pause(&self) -> Result<IpcResponse> {
        self.send_request_with_retry(&IpcRequest::Pause).await
    }

    /// Sends a resume command to the daemon.
    pub async fn resume(&self) -> Result<IpcResponse> {
        self.send_request_with_retry(&IpcRequest::Resume).await
    }

    /// Sends a stop command to the daemon.
    pub async fn stop(&self) -> Result<IpcResponse> {
        self.send_request_with_retry(&IpcRequest::Stop).await
    }

    /// Sends a status query to the daemon.
    pub async fn status(&self) -> Result<IpcResponse> {
        self.send_request_with_retry(&IpcRequest::Status).await
    }

    /// Sends a request to the daemon with retry logic.
    async fn send_request_with_retry(&self, request: &IpcRequest) -> Result<IpcResponse> {
        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            match self.send_request(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!("リクエスト失敗 (試行 {}/{}): {}", attempt, MAX_RETRIES, e);
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        let delay = Duration::from_millis(RETRY_DELAY_MS * u64::from(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Sends a single request to the daemon.
    async fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse> {
        // Connect with timeout
        let mut stream = timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .context("接続がタイムアウトしました")?
            .context("Daemonに接続できません。'pomodoro daemon' を起動してください")?;

        // Serialize request
        let request_json =
            serde_json::to_string(request).context("リクエストのシリアライズに失敗しました")?;

        // Send request with timeout
        timeout(
            Duration::from_secs(IO_TIMEOUT_SECS),
            stream.write_all(request_json.as_bytes()),
        )
        .await
        .context("書き込みがタイムアウトしました")?
        .context("リクエストの送信に失敗しました")?;

        // Flush
        timeout(Duration::from_secs(IO_TIMEOUT_SECS), stream.flush())
            .await
            .context("フラッシュがタイムアウトしました")?
            .context("フラッシュに失敗しました")?;

        // Shutdown write side to signal end of request
        stream
            .shutdown()
            .await
            .context("シャットダウンに失敗しました")?;

        // Read response with timeout
        let mut buffer = vec![0u8; MAX_RESPONSE_SIZE];
        let n = timeout(
            Duration::from_secs(IO_TIMEOUT_SECS),
            stream.read(&mut buffer),
        )
        .await
        .context("読み込みがタイムアウトしました")?
        .context("レスポンスの受信に失敗しました")?;

        if n == 0 {
            anyhow::bail!("Daemonからの応答がありませんでした");
        }

        // Deserialize response
        let response: IpcResponse =
            serde_json::from_slice(&buffer[..n]).context("レスポンスのパースに失敗しました")?;

        // Check for error response
        if response.status == "error" {
            anyhow::bail!("{}", response.message);
        }

        Ok(response)
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new().expect("Failed to create IPC client")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseData;
    use std::sync::Arc;
    use tokio::net::UnixListener;
    use tokio::sync::Mutex;

    // ------------------------------------------------------------------------
    // Helper functions
    // ------------------------------------------------------------------------

    fn create_temp_socket_path() -> PathBuf {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.sock");
        // Keep the directory so it's not deleted
        std::mem::forget(dir);
        path
    }

    async fn create_mock_server(socket_path: &PathBuf) -> UnixListener {
        // Remove existing socket file if present
        let _ = std::fs::remove_file(socket_path);

        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        UnixListener::bind(socket_path).unwrap()
    }

    // ------------------------------------------------------------------------
    // IpcClient Tests
    // ------------------------------------------------------------------------

    mod client_tests {
        use super::*;

        #[test]
        fn test_with_socket_path() {
            let path = PathBuf::from("/tmp/test.sock");
            let client = IpcClient::with_socket_path(path.clone());
            assert_eq!(client.socket_path(), &path);
        }

        #[tokio::test]
        async fn test_connection_failure() {
            let socket_path = PathBuf::from("/tmp/nonexistent_socket_12345.sock");
            let client = IpcClient::with_socket_path(socket_path);

            let result = client.status().await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_send_status_request() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                // Read request
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();

                // Verify it's a status request
                assert!(matches!(request, IpcRequest::Status));

                // Send response
                let response = IpcResponse::success(
                    "",
                    Some(ResponseData {
                        state: Some("stopped".to_string()),
                        remaining_seconds: Some(0),
                        pomodoro_count: Some(0),
                        task_name: None,
                    }),
                );
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
                stream.flush().await.unwrap();
            });

            // Create client and send request
            let client = IpcClient::with_socket_path(socket_path);
            let response = client.status().await.unwrap();

            assert_eq!(response.status, "success");
            assert!(response.data.is_some());

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("stopped".to_string()));

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_send_start_request() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            let received_request = Arc::new(Mutex::new(None));
            let received_clone = received_request.clone();

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                // Read request
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();

                // Store received request
                *received_clone.lock().await = Some(request);

                // Send response
                let response = IpcResponse::success(
                    "タイマーを開始しました",
                    Some(ResponseData {
                        state: Some("working".to_string()),
                        remaining_seconds: Some(1500),
                        pomodoro_count: Some(0),
                        task_name: Some("Test Task".to_string()),
                    }),
                );
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
                stream.flush().await.unwrap();
            });

            // Create client and send request
            let client = IpcClient::with_socket_path(socket_path);
            let args = StartArgs {
                work: 25,
                break_time: 5,
                long_break: 15,
                task: Some("Test Task".to_string()),
                auto_cycle: false,
                focus_mode: false,
                no_sound: false,
            };
            let response = client.start(&args).await.unwrap();

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを開始しました");

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.remaining_seconds, Some(1500));
            assert_eq!(data.task_name, Some("Test Task".to_string()));

            // Verify received request
            let received = received_request.lock().await;
            match received.as_ref() {
                Some(IpcRequest::Start { params }) => {
                    assert_eq!(params.work_minutes, Some(25));
                    assert_eq!(params.break_minutes, Some(5));
                    assert_eq!(params.long_break_minutes, Some(15));
                    assert_eq!(params.task_name, Some("Test Task".to_string()));
                }
                _ => panic!("Expected Start request"),
            }

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_send_pause_request() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                // Read request
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();
                assert!(matches!(request, IpcRequest::Pause));

                // Send response
                let response = IpcResponse::success(
                    "タイマーを一時停止しました",
                    Some(ResponseData {
                        state: Some("paused".to_string()),
                        remaining_seconds: Some(1200),
                        pomodoro_count: Some(0),
                        task_name: None,
                    }),
                );
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
            });

            let client = IpcClient::with_socket_path(socket_path);
            let response = client.pause().await.unwrap();

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを一時停止しました");

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_send_resume_request() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                // Read request
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();
                assert!(matches!(request, IpcRequest::Resume));

                // Send response
                let response = IpcResponse::success(
                    "タイマーを再開しました",
                    Some(ResponseData {
                        state: Some("working".to_string()),
                        remaining_seconds: Some(1200),
                        pomodoro_count: Some(0),
                        task_name: None,
                    }),
                );
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
            });

            let client = IpcClient::with_socket_path(socket_path);
            let response = client.resume().await.unwrap();

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを再開しました");

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_send_stop_request() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                // Read request
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();
                assert!(matches!(request, IpcRequest::Stop));

                // Send response
                let response = IpcResponse::success(
                    "タイマーを停止しました",
                    Some(ResponseData {
                        state: Some("stopped".to_string()),
                        remaining_seconds: Some(0),
                        pomodoro_count: Some(0),
                        task_name: None,
                    }),
                );
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
            });

            let client = IpcClient::with_socket_path(socket_path);
            let response = client.stop().await.unwrap();

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを停止しました");

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_error_response() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            // Spawn mock server that returns error (handles all retry attempts)
            let server_handle = tokio::spawn(async move {
                // Handle up to MAX_RETRIES connections
                for _ in 0..MAX_RETRIES {
                    if let Ok((mut stream, _)) = listener.accept().await {
                        // Read request
                        let mut buffer = vec![0u8; 4096];
                        let _ = stream.read(&mut buffer).await;

                        // Send error response
                        let response = IpcResponse::error("タイマーは既に実行中です");
                        let json = serde_json::to_vec(&response).unwrap();
                        let _ = stream.write_all(&json).await;
                    }
                }
            });

            let client = IpcClient::with_socket_path(socket_path);
            let result = client.start(&StartArgs::default()).await;

            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("既に実行中"),
                "Expected error message to contain '既に実行中', got: {}",
                error_msg
            );

            // Cancel the server task (it may be waiting for more connections)
            server_handle.abort();
        }
    }

    // ------------------------------------------------------------------------
    // StartArgs Conversion Tests
    // ------------------------------------------------------------------------

    mod start_args_tests {
        use super::*;

        #[tokio::test]
        async fn test_start_args_to_params_defaults() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            let received_request = Arc::new(Mutex::new(None));
            let received_clone = received_request.clone();

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();
                *received_clone.lock().await = Some(request);

                let response = IpcResponse::success("OK", None);
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
            });

            let client = IpcClient::with_socket_path(socket_path);
            let _ = client.start(&StartArgs::default()).await;

            let received = received_request.lock().await;
            match received.as_ref() {
                Some(IpcRequest::Start { params }) => {
                    assert_eq!(params.work_minutes, Some(25));
                    assert_eq!(params.break_minutes, Some(5));
                    assert_eq!(params.long_break_minutes, Some(15));
                    assert!(params.task_name.is_none());
                    assert_eq!(params.auto_cycle, Some(false));
                    assert_eq!(params.focus_mode, Some(false));
                }
                _ => panic!("Expected Start request"),
            }

            server_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_start_args_to_params_custom() {
            let socket_path = create_temp_socket_path();
            let listener = create_mock_server(&socket_path).await;

            let received_request = Arc::new(Mutex::new(None));
            let received_clone = received_request.clone();

            // Spawn mock server
            let server_handle = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();

                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let request: IpcRequest = serde_json::from_slice(&buffer[..n]).unwrap();
                *received_clone.lock().await = Some(request);

                let response = IpcResponse::success("OK", None);
                let json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&json).await.unwrap();
            });

            let client = IpcClient::with_socket_path(socket_path);
            let args = StartArgs {
                work: 50,
                break_time: 10,
                long_break: 30,
                task: Some("Custom Task".to_string()),
                auto_cycle: true,
                focus_mode: true,
                no_sound: true,
            };
            let _ = client.start(&args).await;

            let received = received_request.lock().await;
            match received.as_ref() {
                Some(IpcRequest::Start { params }) => {
                    assert_eq!(params.work_minutes, Some(50));
                    assert_eq!(params.break_minutes, Some(10));
                    assert_eq!(params.long_break_minutes, Some(30));
                    assert_eq!(params.task_name, Some("Custom Task".to_string()));
                    assert_eq!(params.auto_cycle, Some(true));
                    assert_eq!(params.focus_mode, Some(true));
                }
                _ => panic!("Expected Start request"),
            }

            server_handle.await.unwrap();
        }
    }
}
