use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZipCodeError {
    #[error("ZIP code not found: {0}")]
    ZipNotFound(String),

    #[error("Invalid ZIP code format: {0}")]
    InvalidZipFormat(String),

    #[error("Invalid coordinates: lat={0}, lon={1}")]
    InvalidCoordinates(f64, f64),

    #[error("Failed to load data: {0}")]
    DataLoadError(String),
}
