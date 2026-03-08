#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[allow(dead_code)]
    #[error("Not found: {0}")]
    NotFound(String),
}
