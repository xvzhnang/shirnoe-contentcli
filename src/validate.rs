use crate::error::AppError;

/// Extension point for Shirone frontmatter and content validation.
pub fn run() -> Result<(), AppError> {
    Err(AppError::Reserved("validate".to_owned()))
}
