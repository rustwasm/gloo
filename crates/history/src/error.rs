use thiserror::Error;

/// The Error type for History.
#[derive(Error, Debug)]
pub enum HistoryError {
    /// Failed to serialize query.
    #[cfg(feature = "query")]
    #[error("failed to serialize query.")]
    QuerySer(#[from] serde_urlencoded::ser::Error),
    /// Failed to deserialize query.
    #[cfg(feature = "query")]
    #[error("failed to deserialize query.")]
    QueryDe(#[from] serde_urlencoded::de::Error),
}

/// The Result type for History.
pub type HistoryResult<T, E = HistoryError> = std::result::Result<T, E>;
