use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("network error: {0}")] Network(String),
    #[error("unauthorized (check API key)")] Unauthorized,
    #[error("not found")] NotFound,
    #[error("rate limited; try later")] RateLimited,
    #[error("invalid input: {0}")] InvalidInput(String),
    #[error("parse error: {0}")] Parse(String),
    #[error("{0}")] Other(String),
}

impl From<reqwest::Error> for UiError {
    fn from(e: reqwest::Error) -> Self { UiError::Network(e.to_string()) }
}
