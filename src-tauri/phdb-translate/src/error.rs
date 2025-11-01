#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    GcpAuth(#[from] gcp_auth::Error),
    #[error(transparent)]
    TranslateApiCall(#[from] reqwest::Error),
    #[error("translate got error response:{0}")]
    TranslateResponse(String),
    #[error("system IO error: {0}")]
    SystemIO(String),
}

pub type Result<T> = std::result::Result<T, Error>;
