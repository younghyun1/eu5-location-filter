//! Application-wide error reporting.

use std::path::PathBuf;

/// Every operational failure exposed by the library or command-line interface.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A filesystem operation failed.
    #[error("failed to {operation} {path}: {source}")]
    Io {
        /// Operation being attempted.
        operation: &'static str,
        /// Affected path.
        path: PathBuf,
        /// Operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// An input parser rejected source data.
    #[error("invalid {source_name} at byte {offset}: {message}")]
    Parse {
        /// Human-readable source name.
        source_name: String,
        /// Byte offset nearest the problem.
        offset: usize,
        /// Specific parsing problem.
        message: String,
    },
    /// Steam or the requested game directory could not be found.
    #[error("Europa Universalis V installation was not found: {0}")]
    Installation(String),
    /// A required installation file is absent.
    #[error("required installation file is missing: {0}")]
    MissingFile(PathBuf),
    /// Imported or stored data violated an invariant.
    #[error("invalid dataset: {0}")]
    InvalidData(String),
    /// Bitcode encoding or decoding failed.
    #[error("dataset encoding failed: {0}")]
    Encoding(String),
    /// Compression or decompression failed.
    #[error("dataset compression failed: {0}")]
    Compression(String),
    /// A PNG could not be decoded.
    #[cfg(feature = "desktop")]
    #[error("PNG decode failed for {path}: {source}")]
    Png {
        /// Affected PNG path.
        path: PathBuf,
        /// Decoder error.
        #[source]
        source: png::DecodingError,
    },
    /// The Slint event loop or UI failed.
    #[error("user interface failed: {0}")]
    Ui(String),
    /// A worker thread exited without returning a result.
    #[error("background import worker terminated unexpectedly")]
    Worker,
}

impl AppError {
    /// Returns whether retrying after user or environmental intervention can succeed.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Io { .. } | Self::Installation(_) | Self::MissingFile(_) | Self::Worker => true,
            #[cfg(feature = "desktop")]
            Self::Png { .. } => true,
            Self::Parse { .. }
            | Self::InvalidData(_)
            | Self::Encoding(_)
            | Self::Compression(_)
            | Self::Ui(_) => false,
        }
    }

    /// Adds path and operation context to an I/O failure.
    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    /// Constructs a parser error at a byte offset.
    pub fn parse(
        source_name: impl Into<String>,
        offset: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::Parse {
            source_name: source_name.into(),
            offset,
            message: message.into(),
        }
    }
}
