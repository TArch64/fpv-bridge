//! Trait abstraction for serial port operations to enable testing

use async_trait::async_trait;
use std::io;

/// Trait for serial port I/O operations
#[async_trait]
pub trait SerialPortIO: Send {
    /// Write all data to the port
    async fn write_all(&mut self, data: &[u8]) -> io::Result<()>;

    /// Flush the output buffer
    async fn flush(&mut self) -> io::Result<()>;
}

/// Wrapper around tokio_serial::SerialStream that implements SerialPortIO
pub struct TokioSerialPort {
    port: tokio_serial::SerialStream,
}

impl TokioSerialPort {
    pub fn new(port: tokio_serial::SerialStream) -> Self {
        Self { port }
    }
}

#[async_trait]
impl SerialPortIO for TokioSerialPort {
    async fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;
        self.port.write_all(data).await
    }

    async fn flush(&mut self) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;
        self.port.flush().await
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock serial port for testing
    #[derive(Clone)]
    pub struct MockSerialPort {
        pub written_data: Arc<Mutex<Vec<Vec<u8>>>>,
        pub write_error: Arc<Mutex<Option<io::ErrorKind>>>,
        pub flush_error: Arc<Mutex<Option<io::ErrorKind>>>,
    }

    impl MockSerialPort {
        pub fn new() -> Self {
            Self {
                written_data: Arc::new(Mutex::new(Vec::new())),
                write_error: Arc::new(Mutex::new(None)),
                flush_error: Arc::new(Mutex::new(None)),
            }
        }

        pub fn get_written_data(&self) -> Vec<Vec<u8>> {
            self.written_data.lock().unwrap().clone()
        }

        pub fn set_write_error(&self, error: io::ErrorKind) {
            *self.write_error.lock().unwrap() = Some(error);
        }

        pub fn set_flush_error(&self, error: io::ErrorKind) {
            *self.flush_error.lock().unwrap() = Some(error);
        }
    }

    #[async_trait]
    impl SerialPortIO for MockSerialPort {
        async fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
            if let Some(error) = *self.write_error.lock().unwrap() {
                return Err(io::Error::new(error, "Mock write error"));
            }
            self.written_data.lock().unwrap().push(data.to_vec());
            Ok(())
        }

        async fn flush(&mut self) -> io::Result<()> {
            if let Some(error) = *self.flush_error.lock().unwrap() {
                return Err(io::Error::new(error, "Mock flush error"));
            }
            Ok(())
        }
    }
}
