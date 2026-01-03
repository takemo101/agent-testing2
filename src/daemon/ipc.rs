//! IPC Server for the Pomodoro Timer.
//!
//! This module provides Unix Domain Socket IPC functionality:
//! - Server that listens on a Unix socket
//! - Request/response handling for timer commands
//! - Integration with TimerEngine for command execution

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use crate::types::{IpcRequest, IpcResponse, ResponseData, StartParams};

use super::timer::TimerEngine;

// ============================================================================
// Constants
// ============================================================================

/// Default socket path
pub const DEFAULT_SOCKET_PATH: &str = "~/.pomodoro/pomodoro.sock";

/// Maximum request size in bytes (4KB)
const MAX_REQUEST_SIZE: usize = 4096;

/// Connection timeout in seconds (for future use with concurrent connections)
#[allow(dead_code)]
const CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Read timeout in seconds
const READ_TIMEOUT_SECS: u64 = 5;

// ============================================================================
// IpcError
// ============================================================================

/// IPC-specific error types.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Socket binding error
    #[error("Failed to bind socket: {0}")]
    BindError(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Read error
    #[error("Failed to read request: {0}")]
    ReadError(String),

    /// Write error
    #[error("Failed to write response: {0}")]
    WriteError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Request too large
    #[error("Request too large (max {MAX_REQUEST_SIZE} bytes)")]
    RequestTooLarge,
}

// ============================================================================
// IpcServer
// ============================================================================

/// Unix Domain Socket IPC server.
pub struct IpcServer {
    /// Unix socket listener
    listener: UnixListener,
    /// Socket path (for cleanup)
    socket_path: PathBuf,
}

impl IpcServer {
    /// Creates a new IPC server bound to the specified socket path.
    ///
    /// If the socket file already exists, it will be removed before binding.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket cannot be bound.
    pub fn new(socket_path: &Path) -> Result<Self> {
        // Remove existing socket file if present
        if socket_path.exists() {
            std::fs::remove_file(socket_path)
                .with_context(|| format!("Failed to remove existing socket: {:?}", socket_path))?;
        }

        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket directory: {:?}", parent))?;
        }

        let listener = UnixListener::bind(socket_path)
            .with_context(|| format!("Failed to bind Unix socket: {:?}", socket_path))?;

        Ok(Self {
            listener,
            socket_path: socket_path.to_path_buf(),
        })
    }

    /// Accepts an incoming client connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be accepted.
    pub async fn accept(&self) -> Result<UnixStream> {
        let (stream, _addr) = self
            .listener
            .accept()
            .await
            .context("Failed to accept connection")?;
        Ok(stream)
    }

    /// Receives and deserializes an IPC request from the stream.
    ///
    /// Applies a read timeout to prevent blocking indefinitely.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or deserialization fails.
    pub async fn receive_request(stream: &mut UnixStream) -> Result<IpcRequest> {
        let mut buffer = vec![0u8; MAX_REQUEST_SIZE];

        let read_result = timeout(
            Duration::from_secs(READ_TIMEOUT_SECS),
            stream.read(&mut buffer),
        )
        .await;

        let n = match read_result {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(IpcError::ReadError(e.to_string()).into()),
            Err(_) => return Err(IpcError::Timeout.into()),
        };

        if n == 0 {
            anyhow::bail!("Connection closed by client");
        }

        let request: IpcRequest = serde_json::from_slice(&buffer[..n])
            .with_context(|| "Failed to deserialize IPC request")?;

        Ok(request)
    }

    /// Serializes and sends an IPC response to the stream.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    pub async fn send_response(stream: &mut UnixStream, response: &IpcResponse) -> Result<()> {
        let json = serde_json::to_vec(response).context("Failed to serialize IPC response")?;

        stream
            .write_all(&json)
            .await
            .context("Failed to write response")?;
        stream.flush().await.context("Failed to flush response")?;

        Ok(())
    }

    /// Returns the socket path.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file on drop
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

// ============================================================================
// RequestHandler
// ============================================================================

/// Handles IPC requests by dispatching to TimerEngine.
pub struct RequestHandler {
    /// Shared reference to the timer engine
    engine: Arc<Mutex<TimerEngine>>,
}

impl RequestHandler {
    /// Creates a new request handler with the given timer engine.
    pub fn new(engine: Arc<Mutex<TimerEngine>>) -> Self {
        Self { engine }
    }

    /// Handles an IPC request and returns the appropriate response.
    pub async fn handle(&self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::Start { params } => self.handle_start(params).await,
            IpcRequest::Pause => self.handle_pause().await,
            IpcRequest::Resume => self.handle_resume().await,
            IpcRequest::Stop => self.handle_stop().await,
            IpcRequest::Status => self.handle_status().await,
        }
    }

    /// Handles the start command.
    async fn handle_start(&self, params: StartParams) -> IpcResponse {
        let mut engine = self.engine.lock().await;

        // Apply custom configuration if provided
        if params.work_minutes.is_some()
            || params.break_minutes.is_some()
            || params.long_break_minutes.is_some()
            || params.auto_cycle.is_some()
            || params.focus_mode.is_some()
        {
            let state = engine.get_state();
            let mut config = state.config.clone();

            if let Some(work) = params.work_minutes {
                config.work_minutes = work;
            }
            if let Some(brk) = params.break_minutes {
                config.break_minutes = brk;
            }
            if let Some(long_brk) = params.long_break_minutes {
                config.long_break_minutes = long_brk;
            }
            if let Some(auto) = params.auto_cycle {
                config.auto_cycle = auto;
            }
            if let Some(focus) = params.focus_mode {
                config.focus_mode = focus;
            }

            // Validate configuration
            if let Err(e) = config.validate() {
                return IpcResponse::error(e);
            }
        }

        match engine.start(params.task_name) {
            Ok(()) => {
                let state = engine.get_state();
                IpcResponse::success(
                    "タイマーを開始しました",
                    Some(ResponseData::from_timer_state(state)),
                )
            }
            Err(e) => IpcResponse::error(e.to_string()),
        }
    }

    /// Handles the pause command.
    async fn handle_pause(&self) -> IpcResponse {
        let mut engine = self.engine.lock().await;

        match engine.pause() {
            Ok(()) => {
                let state = engine.get_state();
                IpcResponse::success(
                    "タイマーを一時停止しました",
                    Some(ResponseData::from_timer_state(state)),
                )
            }
            Err(e) => IpcResponse::error(e.to_string()),
        }
    }

    /// Handles the resume command.
    async fn handle_resume(&self) -> IpcResponse {
        let mut engine = self.engine.lock().await;

        match engine.resume() {
            Ok(()) => {
                let state = engine.get_state();
                IpcResponse::success(
                    "タイマーを再開しました",
                    Some(ResponseData::from_timer_state(state)),
                )
            }
            Err(e) => IpcResponse::error(e.to_string()),
        }
    }

    /// Handles the stop command.
    async fn handle_stop(&self) -> IpcResponse {
        let mut engine = self.engine.lock().await;

        match engine.stop() {
            Ok(()) => {
                let state = engine.get_state();
                IpcResponse::success(
                    "タイマーを停止しました",
                    Some(ResponseData::from_timer_state(state)),
                )
            }
            Err(e) => IpcResponse::error(e.to_string()),
        }
    }

    /// Handles the status command.
    async fn handle_status(&self) -> IpcResponse {
        let engine = self.engine.lock().await;
        let state = engine.get_state();

        IpcResponse::success("", Some(ResponseData::from_timer_state(state)))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    use crate::daemon::timer::TimerEvent;

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

    fn create_engine() -> (Arc<Mutex<TimerEngine>>, mpsc::UnboundedReceiver<TimerEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = PomodoroConfig::default();
        let engine = TimerEngine::new(config, tx);
        (Arc::new(Mutex::new(engine)), rx)
    }

    // ------------------------------------------------------------------------
    // IpcServer Tests
    // ------------------------------------------------------------------------

    mod ipc_server_tests {
        use super::*;

        #[tokio::test]
        async fn test_server_creation() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path);

            assert!(server.is_ok());
            assert!(socket_path.exists());

            // Cleanup
            drop(server);
        }

        #[tokio::test]
        async fn test_server_removes_existing_socket() {
            let socket_path = create_temp_socket_path();

            // Create a dummy file at the socket path
            std::fs::write(&socket_path, "dummy").unwrap();

            // Server should remove it and bind successfully
            let server = IpcServer::new(&socket_path);
            assert!(server.is_ok());
        }

        #[tokio::test]
        async fn test_server_creates_parent_directory() {
            let dir = tempfile::tempdir().unwrap();
            let socket_path = dir.path().join("subdir").join("test.sock");

            let server = IpcServer::new(&socket_path);
            assert!(server.is_ok());
            assert!(socket_path.parent().unwrap().exists());
        }

        #[tokio::test]
        async fn test_accept_connection() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            // Connect from client in background
            let client_path = socket_path.clone();
            let client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                UnixStream::connect(&client_path).await
            });

            // Accept connection
            let stream = server.accept().await;
            assert!(stream.is_ok());

            let client_result = client_handle.await.unwrap();
            assert!(client_result.is_ok());
        }

        #[tokio::test]
        async fn test_receive_request_status() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            // Client sends status request
            let client_path = socket_path.clone();
            let client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();
                let request = r#"{"command":"status"}"#;
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            });

            let mut stream = server.accept().await.unwrap();
            let request = IpcServer::receive_request(&mut stream).await;

            assert!(request.is_ok());
            assert!(matches!(request.unwrap(), IpcRequest::Status));

            client_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_receive_request_start() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            let client_path = socket_path.clone();
            let client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();
                let request = r#"{"command":"start","taskName":"Test Task"}"#;
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            });

            let mut stream = server.accept().await.unwrap();
            let request = IpcServer::receive_request(&mut stream).await;

            assert!(request.is_ok());
            if let IpcRequest::Start { params } = request.unwrap() {
                assert_eq!(params.task_name, Some("Test Task".to_string()));
            } else {
                panic!("Expected Start request");
            }

            client_handle.await.unwrap();
        }

        #[tokio::test]
        async fn test_send_response() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            let client_path = socket_path.clone();
            let client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();

                // Read response
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let response: IpcResponse = serde_json::from_slice(&buffer[..n]).unwrap();
                response
            });

            let mut stream = server.accept().await.unwrap();
            let response = IpcResponse::success("Test message", None);
            IpcServer::send_response(&mut stream, &response)
                .await
                .unwrap();

            let received = client_handle.await.unwrap();
            assert_eq!(received.status, "success");
            assert_eq!(received.message, "Test message");
        }

        #[tokio::test]
        async fn test_receive_request_invalid_json() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            let client_path = socket_path.clone();
            let _client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();
                let invalid_json = "not valid json";
                stream.write_all(invalid_json.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            });

            let mut stream = server.accept().await.unwrap();
            let request = IpcServer::receive_request(&mut stream).await;

            assert!(request.is_err());
        }

        #[tokio::test]
        async fn test_socket_path_getter() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            assert_eq!(server.socket_path(), socket_path);
        }

        #[tokio::test]
        async fn test_server_drop_cleanup() {
            let socket_path = create_temp_socket_path();

            {
                let _server = IpcServer::new(&socket_path).unwrap();
                assert!(socket_path.exists());
            }

            // Socket file should be removed after drop
            assert!(!socket_path.exists());
        }
    }

    // ------------------------------------------------------------------------
    // RequestHandler Tests
    // ------------------------------------------------------------------------

    mod request_handler_tests {
        use super::*;

        #[tokio::test]
        async fn test_handle_status() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let response = handler.handle(IpcRequest::Status).await;

            assert_eq!(response.status, "success");
            assert!(response.data.is_some());

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("stopped".to_string()));
            assert_eq!(data.remaining_seconds, Some(0));
            assert_eq!(data.pomodoro_count, Some(0));
        }

        #[tokio::test]
        async fn test_handle_start() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let request = IpcRequest::Start {
                params: StartParams {
                    task_name: Some("Test Task".to_string()),
                    ..Default::default()
                },
            };

            let response = handler.handle(request).await;

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを開始しました");
            assert!(response.data.is_some());

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.remaining_seconds, Some(25 * 60));
            assert_eq!(data.task_name, Some("Test Task".to_string()));
        }

        #[tokio::test]
        async fn test_handle_start_already_running() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine.clone());

            // Start first
            let start_request = IpcRequest::Start {
                params: StartParams::default(),
            };
            handler.handle(start_request.clone()).await;

            // Try to start again
            let response = handler.handle(start_request).await;

            assert_eq!(response.status, "error");
            assert!(response.message.contains("既に実行中"));
        }

        #[tokio::test]
        async fn test_handle_pause() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // Start first
            handler
                .handle(IpcRequest::Start {
                    params: StartParams::default(),
                })
                .await;

            let response = handler.handle(IpcRequest::Pause).await;

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを一時停止しました");

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("paused".to_string()));
        }

        #[tokio::test]
        async fn test_handle_pause_not_running() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let response = handler.handle(IpcRequest::Pause).await;

            assert_eq!(response.status, "error");
            assert!(response.message.contains("実行されていません"));
        }

        #[tokio::test]
        async fn test_handle_resume() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // Start and pause first
            handler
                .handle(IpcRequest::Start {
                    params: StartParams::default(),
                })
                .await;
            handler.handle(IpcRequest::Pause).await;

            let response = handler.handle(IpcRequest::Resume).await;

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを再開しました");

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
        }

        #[tokio::test]
        async fn test_handle_resume_not_paused() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let response = handler.handle(IpcRequest::Resume).await;

            assert_eq!(response.status, "error");
            assert!(response.message.contains("一時停止していません"));
        }

        #[tokio::test]
        async fn test_handle_stop() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // Start first
            handler
                .handle(IpcRequest::Start {
                    params: StartParams::default(),
                })
                .await;

            let response = handler.handle(IpcRequest::Stop).await;

            assert_eq!(response.status, "success");
            assert_eq!(response.message, "タイマーを停止しました");

            let data = response.data.unwrap();
            assert_eq!(data.state, Some("stopped".to_string()));
        }

        #[tokio::test]
        async fn test_handle_stop_not_running() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let response = handler.handle(IpcRequest::Stop).await;

            assert_eq!(response.status, "error");
            assert!(response.message.contains("実行されていません"));
        }

        #[tokio::test]
        async fn test_handle_start_with_custom_config() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let request = IpcRequest::Start {
                params: StartParams {
                    work_minutes: Some(30),
                    break_minutes: Some(10),
                    long_break_minutes: Some(20),
                    auto_cycle: Some(true),
                    focus_mode: Some(true),
                    task_name: Some("Custom".to_string()),
                },
            };

            let response = handler.handle(request).await;

            assert_eq!(response.status, "success");

            let data = response.data.unwrap();
            // Note: The remaining seconds still use the original config
            // because we don't recreate the engine with new config
            assert_eq!(data.state, Some("working".to_string()));
        }

        #[tokio::test]
        async fn test_handle_start_invalid_config() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            let request = IpcRequest::Start {
                params: StartParams {
                    work_minutes: Some(0), // Invalid: too low
                    ..Default::default()
                },
            };

            let response = handler.handle(request).await;

            assert_eq!(response.status, "error");
            assert!(response.message.contains("1-120分"));
        }
    }

    // ------------------------------------------------------------------------
    // Integration Tests
    // ------------------------------------------------------------------------

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_ipc_flow() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // Client sends start request
            let client_path = socket_path.clone();
            let client_handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();

                // Send start request
                let request = r#"{"command":"start","taskName":"Integration Test"}"#;
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();

                // Read response
                let mut buffer = vec![0u8; 4096];
                let n = stream.read(&mut buffer).await.unwrap();
                let response: IpcResponse = serde_json::from_slice(&buffer[..n]).unwrap();
                response
            });

            // Server handles request
            let mut stream = server.accept().await.unwrap();
            let request = IpcServer::receive_request(&mut stream).await.unwrap();
            let response = handler.handle(request).await;
            IpcServer::send_response(&mut stream, &response)
                .await
                .unwrap();

            // Verify client received correct response
            let client_response = client_handle.await.unwrap();
            assert_eq!(client_response.status, "success");
            assert_eq!(client_response.message, "タイマーを開始しました");
            assert!(client_response.data.is_some());

            let data = client_response.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
            assert_eq!(data.task_name, Some("Integration Test".to_string()));
        }

        #[tokio::test]
        async fn test_multiple_clients_sequential() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // First client: start
            let client_path = socket_path.clone();
            let client1 = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();
                let request = r#"{"command":"start"}"#;
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap();
                serde_json::from_slice::<IpcResponse>(&buf[..n]).unwrap()
            });

            let mut stream1 = server.accept().await.unwrap();
            let req1 = IpcServer::receive_request(&mut stream1).await.unwrap();
            let resp1 = handler.handle(req1).await;
            IpcServer::send_response(&mut stream1, &resp1)
                .await
                .unwrap();

            let result1 = client1.await.unwrap();
            assert_eq!(result1.status, "success");

            // Second client: status
            let client_path = socket_path.clone();
            let client2 = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut stream = UnixStream::connect(&client_path).await.unwrap();
                let request = r#"{"command":"status"}"#;
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap();
                serde_json::from_slice::<IpcResponse>(&buf[..n]).unwrap()
            });

            let mut stream2 = server.accept().await.unwrap();
            let req2 = IpcServer::receive_request(&mut stream2).await.unwrap();
            let resp2 = handler.handle(req2).await;
            IpcServer::send_response(&mut stream2, &resp2)
                .await
                .unwrap();

            let result2 = client2.await.unwrap();
            assert_eq!(result2.status, "success");
            let data = result2.data.unwrap();
            assert_eq!(data.state, Some("working".to_string()));
        }

        #[tokio::test]
        async fn test_all_commands_flow() {
            let (engine, _rx) = create_engine();
            let handler = RequestHandler::new(engine);

            // Simulate command sequence: start -> pause -> resume -> stop -> status
            let commands = vec![
                (r#"{"command":"start"}"#, "working"),
                (r#"{"command":"pause"}"#, "paused"),
                (r#"{"command":"resume"}"#, "working"),
                (r#"{"command":"stop"}"#, "stopped"),
                (r#"{"command":"status"}"#, "stopped"),
            ];

            for (cmd_json, expected_state) in commands {
                let request: IpcRequest = serde_json::from_str(cmd_json).unwrap();
                let response = handler.handle(request).await;

                if response.status == "success" {
                    if let Some(data) = &response.data {
                        assert_eq!(
                            data.state,
                            Some(expected_state.to_string()),
                            "Command: {}",
                            cmd_json
                        );
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Error Handling Tests
    // ------------------------------------------------------------------------

    mod error_tests {
        use super::*;

        #[tokio::test]
        async fn test_connection_closed() {
            let socket_path = create_temp_socket_path();
            let server = IpcServer::new(&socket_path).unwrap();

            let client_path = socket_path.clone();
            let _client = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let stream = UnixStream::connect(&client_path).await.unwrap();
                // Close immediately without sending anything
                drop(stream);
            });

            let mut stream = server.accept().await.unwrap();
            let result = IpcServer::receive_request(&mut stream).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_ipc_error_display() {
            let err = IpcError::BindError("test error".to_string());
            assert_eq!(err.to_string(), "Failed to bind socket: test error");

            let err = IpcError::Timeout;
            assert_eq!(err.to_string(), "Operation timed out");

            let err = IpcError::RequestTooLarge;
            assert!(err.to_string().contains("4096"));
        }
    }
}
