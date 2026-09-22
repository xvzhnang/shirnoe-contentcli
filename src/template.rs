use crate::error::AppError;
use serde_yaml::to_string;
use std::path::Path;
use tera::{Context, Tera};

#[derive(Debug, Clone)]
pub struct PostValues {
    pub slug: String,
    pub title: String,
    pub published: String,
    pub published_at: String,
    pub description: String,
    pub tags: Vec<String>,
    pub category: String,
    pub draft: bool,
    pub comment: bool,
    pub lang: String,
    pub image: String,
}

pub fn render(path: &Path, values: &PostValues) -> Result<String, AppError> {
    let source = std::fs::read_to_string(path)?;
    let mut context = Context::new();
    context.insert("slug", &values.slug);
    context.insert("title", &values.title);
    context.insert("title_yaml", &yaml_scalar(&values.title)?);
    context.insert("published", &values.published);
    context.insert("published_at", &values.published_at);
    context.insert("publishedAt", &values.published_at);
    context.insert("description", &values.description);
    context.insert("description_yaml", &yaml_scalar(&values.description)?);
    context.insert("tags", &values.tags);
    context.insert("tags_yaml", &yaml_tags(&values.tags)?);
    context.insert("category", &values.category);
    context.insert("category_yaml", &yaml_scalar(&values.category)?);
    context.insert("draft", &values.draft);
    context.insert("comment", &values.comment);
    context.insert("lang", &values.lang);
    context.insert("lang_yaml", &yaml_scalar(&values.lang)?);
    context.insert("image", &values.image);
    context.insert("image_yaml", &yaml_scalar(&values.image)?);
    context.insert("cover", &(!values.image.is_empty()));
    let mut tera = Tera::default();
    tera.add_raw_template("post", &source)?;
    Ok(tera.render("post", &context)?)
}

fn yaml_scalar(value: &str) -> Result<String, AppError> {
    Ok(to_string(value)
        .map_err(|e| AppError::Config(e.to_string()))?
        .trim()
        .to_owned())
}

fn yaml_tags(values: &[String]) -> Result<String, AppError> {
    let yaml = to_string(values).map_err(|e| AppError::Config(e.to_string()))?;
    Ok(yaml
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n"))
}
