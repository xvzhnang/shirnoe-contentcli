use crate::error::AppError;

/// Future publisher boundary. Git status, commit and push belong behind this API.
#[allow(dead_code)]
pub trait Publisher {
    fn status(&self) -> Result<(), AppError>;
    fn stage(&self) -> Result<(), AppError>;
    fn commit(&self, message: &str) -> Result<(), AppError>;
    fn push(&self) -> Result<(), AppError>;
}

pub fn run() -> Result<(), AppError> {
    Err(AppError::Reserved("publish".to_owned()))
}
