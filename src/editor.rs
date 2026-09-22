use crate::cli::CreateArgs;
use crate::config::Config;
use crate::error::AppError;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn open_after_create(
    root: &Path,
    config: &Config,
    args: &CreateArgs,
    file: &Path,
) -> Result<(), AppError> {
    if args.no_open || !interactive_terminal() {
        return Ok(());
    }
    let (command, open_file, wait) = match args.editor.as_deref() {
        Some(value) => (value, config.editor.open_file, config.editor.wait),
        None if args.open || config.editor.enabled => (
            config.editor.command.as_str(),
            config.editor.open_file,
            config.editor.wait,
        ),
        None => return Ok(()),
    };
    let confirmed = match confirm(file, command) {
        Ok(value) => value,
        Err(error) => {
            eprintln!(
                "warning: editor confirmation failed: {}; article was created",
                error
            );
            return Ok(());
        }
    };
    if !confirmed {
        return Ok(());
    }
    let spec = match prepare_command(root, command, file, open_file) {
        Ok(spec) => spec,
        Err(error) => {
            eprintln!(
                "warning: editor command could not be prepared: {}; article was created",
                error
            );
            return Ok(());
        }
    };
    let spawn_result = spawn_editor(&spec);
    let mut child = match spawn_result {
        Ok(child) => child,
        Err(source) => {
            eprintln!(
                "warning: failed to start editor `{}`: {}; article was created",
                spec.display, source
            );
            return Ok(());
        }
    };
    if wait {
        match child.wait() {
            Ok(status) if !status.success() => {
                eprintln!(
                    "warning: editor `{}` exited with status {}; article was created",
                    spec.display, status
                );
            }
            Ok(_) => {}
            Err(source) => {
                eprintln!(
                    "warning: failed while waiting for editor `{}`: {}; article was created",
                    spec.display, source
                );
            }
        }
    }
    Ok(())
}

fn spawn_editor(spec: &CommandSpec) -> std::io::Result<std::process::Child> {
    #[cfg(windows)]
    {
        if spec.executable.extension().is_none() {
            let mut cmd_exe = spec.executable.clone();
            cmd_exe.set_extension("cmd");
            if cmd_exe != spec.executable && !cmd_exe.exists() {
                return Command::new(&spec.executable).args(&spec.args).spawn();
            }
            if cmd_exe.exists() {
                return Command::new(cmd_exe).args(&spec.args).spawn();
            }
        }
    }
    Command::new(&spec.executable).args(&spec.args).spawn()
}

fn interactive_terminal() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn confirm(file: &Path, command: &str) -> Result<bool, AppError> {
    eprint!("Open {} with editor `{}`? [Y/n] ", file.display(), command);
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(!matches!(answer.trim(), "n" | "N"))
}

#[derive(Debug, PartialEq, Eq)]
struct CommandSpec {
    executable: PathBuf,
    args: Vec<String>,
    display: String,
}

fn prepare_command(
    root: &Path,
    command: &str,
    file: &Path,
    open_file: bool,
) -> Result<CommandSpec, AppError> {
    let mut words = parse_command(command)?;
    let file_text = absolute_path(file).to_string_lossy().into_owned();
    let mut replaced = false;
    for word in &mut words {
        if word.contains("{file}") {
            *word = word.replace("{file}", &file_text);
            replaced = true;
        }
    }
    if open_file && !replaced {
        words.push(file_text);
    }
    let executable = resolve_executable(root, &words[0]);
    let args = words.into_iter().skip(1).collect::<Vec<_>>();
    let display = std::iter::once(executable.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(CommandSpec {
        executable,
        args,
        display,
    })
}

pub fn parse_command(command: &str) -> Result<Vec<String>, AppError> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut token_started = false;
    let mut quote = None;
    let mut characters = command.chars().peekable();
    while let Some(character) = characters.next() {
        match (quote, character) {
            (Some(q), character) if character == q => quote = None,
            (Some('"'), '\\') => match characters.peek().copied() {
                Some('"') | Some('\\') => current.push(characters.next().unwrap()),
                _ => current.push('\\'),
            },
            (Some(_), character) => current.push(character),
            (None, '"') | (None, '\'') => {
                quote = Some(character);
                token_started = true;
            }
            (None, '\\') => match characters.peek().copied() {
                Some(next)
                    if next.is_whitespace() || next == '\\' || next == '"' || next == '\'' =>
                {
                    token_started = true;
                    current.push(characters.next().unwrap());
                }
                _ => {
                    token_started = true;
                    current.push('\\');
                }
            },
            (None, character) if character.is_whitespace() => {
                if token_started {
                    words.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            (None, character) => {
                token_started = true;
                current.push(character);
            }
        }
    }
    if quote.is_some() {
        return Err(AppError::EditorParse(
            "unterminated quote in editor command".to_owned(),
        ));
    }
    if token_started {
        words.push(current);
    }
    if words.is_empty() || words[0].trim().is_empty() {
        return Err(AppError::EmptyEditorCommand);
    }
    Ok(words)
}

fn resolve_executable(root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() || value.contains('/') || value.contains('\\') {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        }
    } else {
        path.to_path_buf()
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_command, prepare_command, resolve_executable};
    use std::path::Path;

    #[test]
    fn parses_quoted_arguments() {
        assert_eq!(
            parse_command(r#"code --reuse-window "draft notes.md""#).unwrap(),
            vec!["code", "--reuse-window", "draft notes.md"]
        );
    }

    #[test]
    fn rejects_unterminated_quotes() {
        assert!(parse_command("code \"draft.md").is_err());
    }

    #[test]
    fn preserves_empty_quoted_argument() {
        assert_eq!(parse_command("code \"\"").unwrap(), vec!["code", ""]);
    }

    #[test]
    fn appends_file_only_when_enabled() {
        let with_file = prepare_command(
            Path::new("C:/project"),
            "code --reuse-window",
            Path::new("C:/project/content/post/index.md"),
            true,
        )
        .unwrap();
        assert_eq!(with_file.args.len(), 2);
        let without_file = prepare_command(
            Path::new("C:/project"),
            "code --reuse-window",
            Path::new("C:/project/content/post/index.md"),
            false,
        )
        .unwrap();
        assert_eq!(without_file.args, vec!["--reuse-window"]);
    }

    #[test]
    fn replaces_file_placeholder() {
        let spec = prepare_command(
            Path::new("C:/project"),
            "code --goto {file}:1",
            Path::new("C:/project/content/post/index.md"),
            false,
        )
        .unwrap();
        assert!(spec.args[1].contains("index.md:1"));
    }

    #[test]
    fn resolves_relative_executable_from_root_and_keeps_path_commands() {
        assert_eq!(
            resolve_executable(Path::new("C:/project"), "tools/editor.exe"),
            Path::new("C:/project/tools/editor.exe")
        );
        assert_eq!(
            resolve_executable(Path::new("C:/project"), "code"),
            Path::new("code")
        );
    }
}
