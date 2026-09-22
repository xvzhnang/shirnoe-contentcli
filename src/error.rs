use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration file {0} was not found")]
    MissingConfig(PathBuf),
    #[error("invalid slug `{0}`: use relative path segments without `.` or `..`")]
    InvalidSlug(String),
    #[error("target file already exists: {0} (use --force to replace it)")]
    TargetExists(PathBuf),
    #[error("cover file already exists: {0} (use --force-cover to replace it)")]
    CoverExists(PathBuf),
    #[error("template variable rendering failed: {0}")]
    Template(#[from] tera::Error),
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("image decoding failed: {0}")]
    Image(#[from] image::ImageError),
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("editor command parsing failed: {0}")]
    EditorParse(String),
    #[error("editor command is empty")]
    EmptyEditorCommand,
    #[error("operation reserved for a future release: {0}")]
    Reserved(String),
}
