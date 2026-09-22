use crate::error::AppError;
use std::path::{Component, Path, PathBuf};

pub fn validate_slug(slug: &str) -> Result<(), AppError> {
    let path = Path::new(slug);
    if slug.trim().is_empty() || path.is_absolute() {
        return Err(AppError::InvalidSlug(slug.to_owned()));
    }
    for component in path.components() {
        match component {
            Component::Normal(value) if !value.is_empty() => {}
            _ => return Err(AppError::InvalidSlug(slug.to_owned())),
        }
    }
    Ok(())
}

pub fn target_dir(content_dir: &Path, slug: &str) -> PathBuf {
    content_dir.join(Path::new(slug))
}
