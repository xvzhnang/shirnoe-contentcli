use image::{DynamicImage, ImageFormat, RgbImage};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use tempfile::tempdir;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_shirone-content"))
}

fn project() -> tempfile::TempDir {
    let dir = tempdir().expect("temporary project");
    fs::create_dir_all(dir.path().join("templates")).unwrap();
    fs::write(
        dir.path().join("config.toml"),
        r#"
[project]
content_dir = "content/posts"
template = "templates/post.md"
filename = "index.md"
timezone = "Asia/Shanghai"

[defaults]
draft = true
comment = true
description = "default description"
category = "Notes"
tags = ["default"]
lang = "zh-CN"

[cover]
enabled = false
timeout_seconds = 5
filename = "cover.webp"
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("templates/post.md"),
        "---\ntitle: {{ title_yaml }}\npublished: {{ published }}\npublishedAt: {{ published_at }}\ndescription: {{ description_yaml }}\ntags:\n{{ tags_yaml }}\ndraft: {{ draft }}\nimage: {{ image_yaml }}\n---\n\n{{ title }}\n",
    )
    .unwrap();
    dir
}

#[test]
fn creates_nested_post_from_template() {
    let dir = project();
    let output = binary()
        .current_dir(dir.path())
        .args([
            "create",
            "guides/rust-cli",
            "--title",
            "Rust CLI",
            "--tags",
            "Rust,CLI",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let post =
        fs::read_to_string(dir.path().join("content/posts/guides/rust-cli/index.md")).unwrap();
    assert!(post.contains("title: Rust CLI"));
    assert!(post.contains("tags:\n  - Rust\n  - CLI"));
    assert!(post.contains("draft: true"));
}

#[test]
fn yaml_escapes_user_text() {
    let dir = project();
    let output = binary()
        .current_dir(dir.path())
        .args([
            "create",
            "quoted",
            "--title",
            "A \"quoted\" title",
            "--description",
            "line one\nline two",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let post = fs::read_to_string(dir.path().join("content/posts/quoted/index.md")).unwrap();
    assert!(
        post.contains("title: A \"quoted\" title") || post.contains("title: 'A \"quoted\" title'")
    );
    assert!(post.contains("description:"));
}

#[test]
fn dry_run_does_not_write_files() {
    let dir = project();
    let output = binary()
        .current_dir(dir.path())
        .args(["new-post", "preview", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!dir.path().join("content/posts/preview/index.md").exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("target:"));
}

#[test]
fn rejects_traversal_and_existing_post() {
    let dir = project();
    let traversal = binary()
        .current_dir(dir.path())
        .args(["create", "../escape"])
        .output()
        .unwrap();
    assert!(!traversal.status.success());
    let first = binary()
        .current_dir(dir.path())
        .args(["create", "same"])
        .output()
        .unwrap();
    assert!(first.status.success());
    let second = binary()
        .current_dir(dir.path())
        .args(["create", "same"])
        .output()
        .unwrap();
    assert!(!second.status.success());
}

#[test]
fn preserves_shirone_date_in_configured_timezone() {
    let dir = project();
    let output = binary()
        .current_dir(dir.path())
        .args([
            "create",
            "dated",
            "--published",
            "2026-09-22",
            "--published-at",
            "2026-09-22T12:00:00Z",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn downloads_and_converts_cover_to_webp() {
    let dir = project();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0])))
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    let bytes = png.into_inner();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len()).unwrap();
        stream.write_all(&bytes).unwrap();
    });
    let output = binary()
        .current_dir(dir.path())
        .args([
            "create",
            "with-cover",
            "--cover-url",
            &format!("http://{address}/cover"),
        ])
        .output()
        .unwrap();
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cover = dir.path().join("content/posts/with-cover/cover.webp");
    assert!(cover.is_file());
    assert_eq!(&fs::read(&cover).unwrap()[0..4], b"RIFF");
    assert!(
        fs::read_to_string(dir.path().join("content/posts/with-cover/index.md"))
            .unwrap()
            .contains("image: ./cover.webp")
    );
}

#[test]
fn enabled_editor_is_skipped_in_noninteractive_runs() {
    let dir = project();
    fs::write(
        dir.path().join("config.toml"),
        fs::read_to_string(dir.path().join("config.toml")).unwrap()
            + "\n[editor]\nenabled = true\ncommand = \"missing-editor\"\nopen_file = true\nwait = false\n",
    )
    .unwrap();
    let output = binary()
        .current_dir(dir.path())
        .args(["create", "noninteractive"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        dir.path()
            .join("content/posts/noninteractive/index.md")
            .is_file()
    );
}

#[test]
fn invalid_enabled_editor_command_is_rejected_before_write() {
    let dir = project();
    fs::write(
        dir.path().join("config.toml"),
        fs::read_to_string(dir.path().join("config.toml")).unwrap()
            + "\n[editor]\nenabled = true\ncommand = \"code \\\"draft\"\n",
    )
    .unwrap();
    let output = binary()
        .current_dir(dir.path())
        .args(["create", "invalid-editor"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        !dir.path()
            .join("content/posts/invalid-editor/index.md")
            .exists()
    );
}
