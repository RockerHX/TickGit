use std::{path::Path, process::Command};

use crate::error::{AppError, AppResult};

#[derive(Clone, Copy)]
enum OutputMode {
    // 只关心命令成功/失败，不消费 stdout 文本。
    Command,
    // 保留完整文本，适合 diff 这类需要保留换行的输出。
    Text,
    // 返回裁剪后的纯文本，适合分支名、计数、hash 列表等解析场景。
    TrimmedText,
}

pub(super) fn git_output_bytes(repo_path: &Path, args: &[&str]) -> AppResult<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if !stderr.is_empty() { stderr } else { stdout };

    Err(AppError::new(
        "git_command_failed",
        if message.is_empty() {
            "Git 命令执行失败".to_string()
        } else {
            message
        },
    ))
}

fn git_command(repo_path: &Path, mode: OutputMode, args: &[&str]) -> AppResult<String> {
    let mut command = Command::new("git");

    if matches!(mode, OutputMode::Text | OutputMode::TrimmedText) {
        command.arg("--no-pager").arg("-c").arg("color.ui=false");
    }

    let output = command
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Ok(match mode {
            OutputMode::TrimmedText => stdout.trim().to_string(),
            OutputMode::Command | OutputMode::Text => stdout,
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if !stderr.is_empty() { stderr } else { stdout };

    Err(AppError::new(
        "git_command_failed",
        if message.is_empty() {
            "Git 命令执行失败".to_string()
        } else {
            message
        },
    ))
}

pub(super) fn git_text(repo_path: &Path, args: &[&str]) -> AppResult<String> {
    git_command(repo_path, OutputMode::Text, args)
}

pub(super) fn git_trimmed(repo_path: &Path, args: &[&str]) -> AppResult<String> {
    git_command(repo_path, OutputMode::TrimmedText, args)
}

pub(super) fn git_run(repo_path: &Path, args: &[&str]) -> AppResult<()> {
    let _ = git_command(repo_path, OutputMode::Command, args)?;
    Ok(())
}
