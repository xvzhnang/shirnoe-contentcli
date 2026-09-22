use crate::error::AppError;

/// Extension point for frontmatter-preserving updates.
pub fn run(_slug: &str) -> Result<(), AppError> {
    Err(AppError::Reserved("update".to_owned()))
}
