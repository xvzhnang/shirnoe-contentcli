use crate::cli::CreateArgs;
use crate::config::Config;
use crate::cover;
use crate::editor;
use crate::error::AppError;
use crate::project::{target_dir, validate_slug};
use crate::template::{self, PostValues};
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn run(root: &Path, config: &Config, args: &CreateArgs) -> Result<(), AppError> {
    validate_slug(&args.slug)?;
    let content_dir = args
        .content_dir
        .as_deref()
        .unwrap_or(&config.project.content_dir);
    let content_dir = Config::resolve_path(root, content_dir);
    let template_path = args.template.as_deref().unwrap_or(&config.project.template);
    let template_path = Config::resolve_path(root, template_path);
    let filename = args.filename.as_deref().unwrap_or(&config.project.filename);
    config.validate()?;
    if let Some(command) = args.editor.as_deref() {
        editor::parse_command(command)?;
    }
    if args.open && args.editor.is_none() && config.editor.command.trim().is_empty() {
        return Err(AppError::Config(
            "editor.command must not be empty when --open is used".to_owned(),
        ));
    }
    if args.open && args.editor.is_none() {
        editor::parse_command(&config.editor.command)?;
    }
    if Path::new(filename).file_name().and_then(|v| v.to_str()) != Some(filename)
        || filename.is_empty()
    {
        return Err(AppError::Config(
            "filename must be a single non-empty file name".to_owned(),
        ));
    }

    let directory = target_dir(&content_dir, &args.slug);
    let markdown_path = directory.join(filename);
    if markdown_path.exists() && !args.force {
        return Err(AppError::TargetExists(markdown_path));
    }

    let (published, published_at) = publication_values(config, args)?;
    let title = args.title.clone().unwrap_or_else(|| {
        Path::new(&args.slug)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(&args.slug)
            .replace(['-', '_'], " ")
    });
    let should_cover = if args.no_cover {
        false
    } else {
        args.cover || args.cover_url.is_some() || config.cover.enabled
    };
    let cover_url = args.cover_url.clone().or_else(|| {
        config
            .cover
            .endpoint
            .clone()
            .filter(|value| !value.trim().is_empty())
    });
    if should_cover && cover_url.is_none() {
        return Err(AppError::Config(
            "cover is enabled but cover.endpoint is empty".to_owned(),
        ));
    }
    let cover_bytes = if should_cover {
        Some(cover::download_webp(
            cover_url.as_deref().unwrap(),
            &config.cover,
        )?)
    } else {
        None
    };
    let image = cover_bytes
        .as_ref()
        .map(|_| format!("./{}", config.cover.filename))
        .unwrap_or_default();
    let values = PostValues {
        slug: args.slug.clone(),
        title,
        published,
        published_at,
        image,
        description: args
            .description
            .clone()
            .unwrap_or_else(|| config.defaults.description.clone()),
        tags: args
            .tags
            .clone()
            .unwrap_or_else(|| config.defaults.tags.clone()),
        category: args
            .category
            .clone()
            .unwrap_or_else(|| config.defaults.category.clone()),
        draft: args.draft.unwrap_or(config.defaults.draft),
        comment: args.comment.unwrap_or(config.defaults.comment),
        lang: args
            .lang
            .clone()
            .unwrap_or_else(|| config.defaults.lang.clone()),
    };
    let markdown = template::render(&template_path, &values)?;
    let cover_path = directory.join(&config.cover.filename);
    let previous_cover = if cover_bytes.is_some() && cover_path.is_file() {
        Some(fs::read(&cover_path)?)
    } else {
        None
    };
    if cover_bytes.is_some() && cover_path.exists() && !args.force_cover {
        return Err(AppError::CoverExists(cover_path));
    }
    if args.dry_run {
        println!("target: {}", markdown_path.display());
        println!("template: {}", template_path.display());
        if should_cover {
            println!("cover: {}", cover_path.display());
        }
        print!("\n{}", markdown);
        return Ok(());
    }
    fs::create_dir_all(&directory)?;
    if let Some(bytes) = cover_bytes {
        atomic_write(&cover_path, &bytes, args.force_cover)?;
    }
    if let Err(error) = atomic_write(&markdown_path, markdown.as_bytes(), args.force) {
        if should_cover {
            match previous_cover {
                Some(previous) => {
                    let _ = atomic_write(&cover_path, &previous, true);
                }
                None => {
                    let _ = fs::remove_file(&cover_path);
                }
            }
        }
        return Err(error);
    }
    println!("created {}", markdown_path.display());
    if should_cover {
        println!("created {}", cover_path.display());
    }
    editor::open_after_create(root, config, args, &markdown_path)?;
    Ok(())
}

fn publication_values(config: &Config, args: &CreateArgs) -> Result<(String, String), AppError> {
    let tz: Tz =
        config.project.timezone.parse().map_err(|_| {
            AppError::Config(format!("invalid timezone `{}`", config.project.timezone))
        })?;
    let now = Utc::now().with_timezone(&tz);
    let published_at = args
        .published_at
        .clone()
        .unwrap_or_else(|| now.to_rfc3339());
    let published = args
        .published
        .clone()
        .unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
    NaiveDate::parse_from_str(&published, "%Y-%m-%d")
        .map_err(|e| AppError::Config(format!("invalid published date: {e}")))?;
    let parsed: DateTime<Utc> = published_at
        .parse()
        .map_err(|e| AppError::Config(format!("invalid published-at value: {e}")))?;
    let local_date = parsed.with_timezone(&tz).format("%Y-%m-%d").to_string();
    if local_date != published {
        return Err(AppError::Config(
            "published and published-at must be the same date in the configured timezone"
                .to_owned(),
        ));
    }
    Ok((published, published_at))
}

fn atomic_write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), AppError> {
    if path.exists() && !overwrite {
        return Err(AppError::TargetExists(path.to_path_buf()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Config("target has no parent directory".to_owned()))?;
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    temp.persist(path).map_err(|e| AppError::Io(e.error))?;
    Ok(())
}
