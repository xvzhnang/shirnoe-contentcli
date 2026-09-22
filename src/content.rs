use crate::error::AppError;

/// Future content-separation backends will implement these operations.
pub fn run(operation: &str) -> Result<(), AppError> {
    Err(AppError::Reserved(format!("content {operation}")))
}
