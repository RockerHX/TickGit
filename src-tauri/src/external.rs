use std::{path::Path, process::Command};

use crate::{
    error::{AppError, AppResult},
    git,
};

const EXTERNAL_TOOL_UNAVAILABLE: &str = "external_tool_unavailable";

fn external_tool_error(message: impl Into<String>) -> AppError {
    AppError::new(EXTERNAL_TOOL_UNAVAILABLE, message)
}

fn spawn_command(command: &mut Command, error_message: &str) -> AppResult<()> {
    command
        .spawn()
        .map(|_| ())
        .map_err(|_| external_tool_error(error_message))
}

fn run_first_available(commands: Vec<Command>, error_message: &str) -> AppResult<()> {
    for mut command in commands {
        if command.spawn().is_ok() {
            return Ok(());
        }
    }

    Err(external_tool_error(error_message))
}

fn command(program: &str) -> Command {
    Command::new(program)
}

pub fn parse_github_remote_url(remote_url: &str) -> Option<String> {
    let trimmed = remote_url.trim();
    let without_suffix = trimmed.strip_suffix(".git").unwrap_or(trimmed);

    if let Some(path) = without_suffix.strip_prefix("https://github.com/") {
        return github_web_url_from_path(path);
    }

    if let Some(path) = without_suffix.strip_prefix("http://github.com/") {
        return github_web_url_from_path(path);
    }

    if let Some(path) = without_suffix.strip_prefix("git@github.com:") {
        return github_web_url_from_path(path);
    }

    if let Some(path) = without_suffix.strip_prefix("ssh://git@github.com/") {
        return github_web_url_from_path(path);
    }

    None
}

fn github_web_url_from_path(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();

    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }

    Some(format!("https://github.com/{owner}/{repo}"))
}

fn origin_remote_url(repo_path: &Path) -> AppResult<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["remote", "get-url", "origin"])
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!remote_url.is_empty()).then_some(remote_url));
    }

    Ok(None)
}

pub fn get_repository_github_url(repo_path: &str) -> AppResult<Option<String>> {
    let repo_path = git::resolve_repository_path(repo_path)?;
    Ok(origin_remote_url(&repo_path)?.and_then(|url| parse_github_remote_url(&url)))
}

pub fn reveal_repository_in_file_manager(repo_path: &str) -> AppResult<()> {
    let repo_path = git::resolve_repository_path(repo_path)?;
    reveal_path(&repo_path)
}

pub fn open_terminal_at_repository(repo_path: &str) -> AppResult<()> {
    let repo_path = git::resolve_repository_path(repo_path)?;
    open_terminal(&repo_path)
}

pub fn open_repository_in_vscode(repo_path: &str) -> AppResult<()> {
    let repo_path = git::resolve_repository_path(repo_path)?;
    open_vscode(&repo_path)
}

#[cfg(target_os = "macos")]
fn reveal_path(repo_path: &Path) -> AppResult<()> {
    spawn_command(
        command("open").arg("-R").arg(repo_path),
        "无法在 Finder 中显示仓库，请确认 Finder 可用",
    )
}

#[cfg(target_os = "windows")]
fn reveal_path(repo_path: &Path) -> AppResult<()> {
    let selector = format!("/select,{}", repo_path.display());
    spawn_command(
        command("explorer").arg(selector),
        "无法在 Explorer 中显示仓库，请确认文件管理器可用",
    )
}

#[cfg(all(unix, not(target_os = "macos")))]
fn reveal_path(repo_path: &Path) -> AppResult<()> {
    spawn_command(
        command("xdg-open").arg(repo_path),
        "无法打开文件管理器，请确认 xdg-open 可用",
    )
}

#[cfg(target_os = "macos")]
fn open_terminal(repo_path: &Path) -> AppResult<()> {
    spawn_command(
        command("open").arg("-a").arg("Terminal").arg(repo_path),
        "无法打开 Terminal，请确认系统已安装可用终端",
    )
}

#[cfg(target_os = "windows")]
fn open_terminal(repo_path: &Path) -> AppResult<()> {
    let mut wt = command("wt");
    wt.arg("-d").arg(repo_path);

    let location = format!(
        "Set-Location -LiteralPath '{}'",
        repo_path.display().to_string().replace('\'', "''")
    );
    let mut powershell = command("powershell");
    powershell.args([
        "-NoProfile",
        "-Command",
        "Start-Process",
        "powershell",
        "-ArgumentList",
        &format!("'-NoExit','-Command','{location}'"),
    ]);

    run_first_available(
        vec![wt, powershell],
        "无法打开 Terminal，请确认系统已安装可用终端",
    )
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_terminal(repo_path: &Path) -> AppResult<()> {
    let mut gnome = command("gnome-terminal");
    gnome.arg("--working-directory").arg(repo_path);

    let mut konsole = command("konsole");
    konsole.arg("--workdir").arg(repo_path);

    let mut xfce = command("xfce4-terminal");
    xfce.arg("--working-directory").arg(repo_path);

    let mut xterm = command("xterm");
    xterm
        .arg("-e")
        .arg(format!("cd '{}' && exec $SHELL", repo_path.display()));

    run_first_available(
        vec![gnome, konsole, xfce, xterm],
        "无法打开 Terminal，请确认系统已安装可用终端",
    )
}

#[cfg(target_os = "macos")]
fn open_vscode(repo_path: &Path) -> AppResult<()> {
    let mut code = command("code");
    code.arg(repo_path);

    let mut app = command("open");
    app.arg("-a").arg("Visual Studio Code").arg(repo_path);

    run_first_available(
        vec![code, app],
        "无法打开 Visual Studio Code，请确认已安装 VS Code 或 code 命令",
    )
}

#[cfg(target_os = "windows")]
fn open_vscode(repo_path: &Path) -> AppResult<()> {
    spawn_command(
        command("code").arg(repo_path),
        "无法打开 Visual Studio Code，请确认已安装 VS Code 或 code 命令",
    )
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_vscode(repo_path: &Path) -> AppResult<()> {
    spawn_command(
        command("code").arg(repo_path),
        "无法打开 Visual Studio Code，请确认已安装 VS Code 或 code 命令",
    )
}

#[cfg(test)]
mod tests {
    use super::parse_github_remote_url;

    #[test]
    fn parses_github_https_remote_url() {
        assert_eq!(
            parse_github_remote_url("https://github.com/openai/codex.git"),
            Some("https://github.com/openai/codex".to_string())
        );
    }

    #[test]
    fn parses_github_scp_remote_url() {
        assert_eq!(
            parse_github_remote_url("git@github.com:openai/codex.git"),
            Some("https://github.com/openai/codex".to_string())
        );
    }

    #[test]
    fn parses_github_ssh_remote_url() {
        assert_eq!(
            parse_github_remote_url("ssh://git@github.com/openai/codex.git"),
            Some("https://github.com/openai/codex".to_string())
        );
    }

    #[test]
    fn ignores_non_github_and_empty_remote_urls() {
        assert_eq!(parse_github_remote_url("https://example.com/a/b.git"), None);
        assert_eq!(parse_github_remote_url(""), None);
    }
}
