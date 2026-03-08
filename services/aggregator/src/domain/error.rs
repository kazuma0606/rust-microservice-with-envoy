#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
