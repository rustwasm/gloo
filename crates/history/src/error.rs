use thiserror::Error;

/// The Error type for History.
#[derive(Error, Debug)]
pub enum HistoryError {
    /// Failed to serialize / deserialize state.
    #[cfg(feature = "state")]
    #[error("failed to serialize / deserialize state.")]
    State(#[from] serde_wasm_bindgen::Error),
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
pub type HistoryResult<T> = std::result::Result<T, HistoryError>;
