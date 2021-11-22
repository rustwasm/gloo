use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("failed to serialize / deserialize state.")]
    State(#[from] serde_wasm_bindgen::Error),
    #[error("failed to serialize query.")]
    QuerySer(#[from] serde_urlencoded::ser::Error),
    #[error("failed to deserialize query.")]
    QueryDe(#[from] serde_urlencoded::de::Error),
}

pub type HistoryResult<T> = std::result::Result<T, HistoryError>;
