use std::{path::Path, process::Command, time::Instant};

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

fn perf_debug_enabled() -> bool {
    std::env::var("TICKGIT_PERF").is_ok_and(|value| value == "1")
}

fn log_git_perf(
    mode: &str,
    args: &[&str],
    started_at: Instant,
    status_code: Option<i32>,
    stdout_bytes: usize,
) {
    if !perf_debug_enabled() {
        return;
    }

    let command = args.first().copied().unwrap_or("unknown");
    eprintln!(
        "[tickgit:perf] git mode={mode} command={command} args={} status={} stdout_bytes={} elapsed_ms={:.1}",
        args.len(),
        status_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string()),
        stdout_bytes,
        started_at.elapsed().as_secs_f64() * 1000.0,
    );
}

pub(super) fn git_output_bytes(repo_path: &Path, args: &[&str]) -> AppResult<Vec<u8>> {
    let started_at = Instant::now();
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
    log_git_perf(
        "bytes",
        args,
        started_at,
        output.status.code(),
        output.stdout.len(),
    );

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

fn git_command_with_allowed_exit_codes(
    repo_path: &Path,
    mode: OutputMode,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> AppResult<String> {
    let mut command = Command::new("git");
    let started_at = Instant::now();

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
    log_git_perf(
        match mode {
            OutputMode::Command => "command",
            OutputMode::Text => "text",
            OutputMode::TrimmedText => "trimmed",
        },
        args,
        started_at,
        output.status.code(),
        output.stdout.len(),
    );

    if output.status.success()
        || output
            .status
            .code()
            .is_some_and(|code| allowed_exit_codes.contains(&code))
    {
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

fn git_command(repo_path: &Path, mode: OutputMode, args: &[&str]) -> AppResult<String> {
    git_command_with_allowed_exit_codes(repo_path, mode, args, &[])
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

pub(super) fn git_text_allow_exit_code(
    repo_path: &Path,
    args: &[&str],
    allowed_exit_code: i32,
) -> AppResult<String> {
    git_command_with_allowed_exit_codes(repo_path, OutputMode::Text, args, &[allowed_exit_code])
}

pub(super) fn git_trimmed_allow_exit_code(
    repo_path: &Path,
    args: &[&str],
    allowed_exit_code: i32,
) -> AppResult<String> {
    git_command_with_allowed_exit_codes(
        repo_path,
        OutputMode::TrimmedText,
        args,
        &[allowed_exit_code],
    )
}
