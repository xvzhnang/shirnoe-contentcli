use crate::error::AppError;
use chrono_tz::Tz;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub project: ProjectConfig,
    pub defaults: DefaultsConfig,
    pub cover: CoverConfig,
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub content_dir: PathBuf,
    pub template: PathBuf,
    pub filename: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    pub draft: bool,
    pub comment: bool,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub lang: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CoverConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub timeout_seconds: u64,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub enabled: bool,
    pub command: String,
    #[serde(default = "default_open_file")]
    pub open_file: bool,
    pub wait: bool,
}

fn default_open_file() -> bool {
    true
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            command: String::new(),
            open_file: true,
            wait: false,
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            content_dir: PathBuf::from("src/content/posts"),
            template: PathBuf::from("templates/post.md"),
            filename: "index.md".to_owned(),
            timezone: "Asia/Shanghai".to_owned(),
        }
    }
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            draft: true,
            comment: true,
            description: String::new(),
            category: String::new(),
            tags: Vec::new(),
            lang: String::new(),
        }
    }
}

impl Default for CoverConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            timeout_seconds: 20,
            filename: "cover.webp".to_owned(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_filename("project.filename", &self.project.filename)?;
        validate_filename("cover.filename", &self.cover.filename)?;
        self.project.timezone.parse::<Tz>().map_err(|_| {
            AppError::Config(format!("invalid timezone `{}`", self.project.timezone))
        })?;
        if self.editor.enabled {
            if self.editor.command.trim().is_empty() {
                return Err(AppError::Config(
                    "editor.command must not be empty when editor.enabled is true".to_owned(),
                ));
            }
            crate::editor::parse_command(&self.editor.command)
                .map_err(|error| AppError::Config(format!("invalid editor command: {error}")))?;
        }
        Ok(())
    }
}

fn validate_filename(name: &str, value: &str) -> Result<(), AppError> {
    if value.is_empty() || Path::new(value).file_name().and_then(|v| v.to_str()) != Some(value) {
        return Err(AppError::Config(format!(
            "{name} must be a single non-empty file name"
        )));
    }
    Ok(())
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::MissingConfig(path.to_path_buf()));
        }
        let source = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&source).map_err(|error| {
            let message = error.to_string();
            if message.contains("invalid unicode") || message.contains("invalid escape") {
                AppError::Config(format!(
                    "{message}\nWindows paths in TOML must use single quotes, forward slashes, or escaped backslashes; for example: content_dir = 'C:\\Users\\name\\content'"
                ))
            } else {
                AppError::Config(message)
            }
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn resolve_path(root: &Path, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn editor_open_file_defaults_to_true_when_section_is_partial() {
        let config: Config = toml::from_str(
            r#"
            [editor]
            enabled = true
            command = "code"
            "#,
        )
        .unwrap();
        assert!(config.editor.open_file);
        assert!(!config.editor.wait);
    }

    #[test]
    fn enabled_editor_requires_a_valid_command() {
        let config: Config = toml::from_str(
            r#"
            [editor]
            enabled = true
            command = "code \"draft"
            "#,
        )
        .unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn windows_path_parse_error_includes_actionable_hint() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            "[project]\ncontent_dir = \"C:\\Users\\name\\content\"\n",
        )
        .unwrap();
        let error = Config::load(&path).unwrap_err().to_string();
        assert!(error.contains("single quotes"));
        assert!(error.contains("forward slashes"));
    }
}
