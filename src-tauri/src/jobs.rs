use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use tauri::{AppHandle, Emitter, State};

use crate::{
    error::{AppError, AppResult},
    git,
    models::{
        PushTargetKind, PushToCommitFailed, PushToCommitFinished, PushToCommitJobStarted,
        PushToCommitProgress, PushToCommitRequest, StepPushFailed, StepPushFinished,
        StepPushJobStarted, StepPushProgress, StepPushRequest,
    },
};

pub const STEP_PUSH_PROGRESS_EVENT: &str = "step-push-progress";
pub const STEP_PUSH_FINISHED_EVENT: &str = "step-push-finished";
pub const STEP_PUSH_FAILED_EVENT: &str = "step-push-failed";
pub const PUSH_TO_COMMIT_FINISHED_EVENT: &str = "push-to-commit-finished";
pub const PUSH_TO_COMMIT_PROGRESS_EVENT: &str = "push-to-commit-progress";
pub const PUSH_TO_COMMIT_FAILED_EVENT: &str = "push-to-commit-failed";
const REMOTE_NAME: &str = "origin";

pub struct StepPushManager {
    next_job_id: AtomicU64,
    running_job: Arc<Mutex<Option<u64>>>,
}

impl StepPushManager {
    pub fn new() -> Self {
        Self {
            next_job_id: AtomicU64::new(1),
            running_job: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct PushToCommitManager {
    next_job_id: AtomicU64,
    running_job: Arc<Mutex<Option<u64>>>,
}

impl PushToCommitManager {
    pub fn new() -> Self {
        Self {
            next_job_id: AtomicU64::new(1),
            running_job: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct PushExecutionGate {
    running_task: Arc<Mutex<Option<String>>>,
}

impl PushExecutionGate {
    pub fn new() -> Self {
        Self {
            running_task: Arc::new(Mutex::new(None)),
        }
    }
}

fn validate_non_empty(value: &str, code: &str, message: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::new(code, message));
    }

    Ok(())
}

fn validate_step_push_request_shape(request: &StepPushRequest) -> AppResult<()> {
    validate_non_empty(&request.repo_path, "invalid_repository", "仓库路径不能为空")?;
    validate_non_empty(&request.branch, "invalid_branch", "目标分支不能为空")?;

    if request.hashes.is_empty() {
        return Err(AppError::new("empty_hashes", "没有可推送的 Commit"));
    }

    for hash in &request.hashes {
        validate_non_empty(hash, "invalid_hash", "目标 Commit 不能为空")?;
    }

    Ok(())
}

fn emit_push_to_commit_progress(
    app: &AppHandle,
    job_id: u64,
    target: &str,
    target_kind: PushTargetKind,
    status: &str,
) {
    let _ = app.emit(
        PUSH_TO_COMMIT_PROGRESS_EVENT,
        PushToCommitProgress {
            job_id,
            target: target.to_string(),
            target_kind,
            status: status.to_string(),
        },
    );
}

fn emit_push_to_commit_failed(
    app: &AppHandle,
    job_id: u64,
    target: &str,
    target_kind: PushTargetKind,
    error: AppError,
) {
    let _ = app.emit(
        PUSH_TO_COMMIT_FAILED_EVENT,
        PushToCommitFailed {
            job_id,
            target: target.to_string(),
            target_kind,
            message: error.message,
            code: error.code,
        },
    );
}

fn clear_running_job(running_job: &Arc<Mutex<Option<u64>>>, job_id: u64) {
    if let Ok(mut current) = running_job.lock() {
        if current.as_ref() == Some(&job_id) {
            *current = None;
        }
    }
}

fn reserve_push_execution(
    running_task: &Arc<Mutex<Option<String>>>,
    task_key: &str,
) -> AppResult<()> {
    let mut current = running_task.lock().expect("push execution gate poisoned");
    if current.is_some() {
        return Err(AppError::new(
            "push_busy",
            "已有推送任务正在运行，请等待完成",
        ));
    }
    *current = Some(task_key.to_string());
    Ok(())
}

fn clear_push_execution(running_task: &Arc<Mutex<Option<String>>>, task_key: &str) {
    if let Ok(mut current) = running_task.lock() {
        if current.as_deref() == Some(task_key) {
            *current = None;
        }
    }
}

pub fn start_step_push(
    app: AppHandle,
    jobs: State<'_, StepPushManager>,
    gate: State<'_, PushExecutionGate>,
    request: StepPushRequest,
) -> AppResult<StepPushJobStarted> {
    validate_step_push_request_shape(&request)?;

    let job_id = jobs.next_job_id.fetch_add(1, Ordering::SeqCst);
    let running_job = Arc::clone(&jobs.running_job);
    let task_key = format!("step-push:{job_id}");
    let running_task = Arc::clone(&gate.running_task);

    reserve_push_execution(&running_task, &task_key)?;

    {
        let mut current = running_job.lock().expect("step push state poisoned");
        if current.is_some() {
            clear_push_execution(&running_task, &task_key);
            return Err(AppError::new(
                "step_push_busy",
                "已有分步提交任务正在运行，请等待完成",
            ));
        }
        *current = Some(job_id);
    }

    let total = request.hashes.len();
    let repo_path = request.repo_path.clone();
    let requested_branch = request.branch.clone();
    let hashes = request.hashes.clone();
    let first_hash = hashes[0].clone();
    let delay_ms = request.delay_ms.unwrap_or(1500);
    let task_key_for_thread = task_key.clone();

    thread::spawn(move || {
        let _ = app.emit(
            STEP_PUSH_PROGRESS_EVENT,
            StepPushProgress {
                job_id,
                current: 0,
                total,
                hash: first_hash.clone(),
                status: "preparing".to_string(),
            },
        );

        let branch = match git::validate_current_branch(&repo_path, &requested_branch) {
            Ok(branch) => branch,
            Err(error) => {
                let _ = app.emit(
                    STEP_PUSH_FAILED_EVENT,
                    StepPushFailed {
                        job_id,
                        current: 0,
                        total,
                        hash: first_hash.clone(),
                        message: error.message,
                        code: error.code,
                    },
                );
                clear_running_job(&running_job, job_id);
                clear_push_execution(&running_task, &task_key_for_thread);
                return;
            }
        };

        if let Err(error) = git::validate_step_push_hashes(&repo_path, &hashes) {
            let _ = app.emit(
                STEP_PUSH_FAILED_EVENT,
                StepPushFailed {
                    job_id,
                    current: 0,
                    total,
                    hash: first_hash.clone(),
                    message: error.message,
                    code: error.code,
                },
            );
            clear_running_job(&running_job, job_id);
            clear_push_execution(&running_task, &task_key_for_thread);
            return;
        }

        for (index, hash) in hashes.iter().enumerate() {
            if let Err(error) = git::push_to_commit_prechecked(&repo_path, &branch, hash) {
                let _ = app.emit(
                    STEP_PUSH_FAILED_EVENT,
                    StepPushFailed {
                        job_id,
                        current: index + 1,
                        total,
                        hash: hash.clone(),
                        message: error.message,
                        code: error.code,
                    },
                );
                clear_running_job(&running_job, job_id);
                clear_push_execution(&running_task, &task_key_for_thread);
                return;
            }

            let _ = app.emit(
                STEP_PUSH_PROGRESS_EVENT,
                StepPushProgress {
                    job_id,
                    current: index + 1,
                    total,
                    hash: hash.clone(),
                    status: "running".to_string(),
                },
            );

            if index + 1 < total {
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }

        let _ = app.emit(STEP_PUSH_FINISHED_EVENT, StepPushFinished { job_id, total });
        clear_running_job(&running_job, job_id);
        clear_push_execution(&running_task, &task_key_for_thread);
    });

    Ok(StepPushJobStarted { job_id, total })
}

pub fn start_push_to_commit(
    app: AppHandle,
    jobs: State<'_, PushToCommitManager>,
    gate: State<'_, PushExecutionGate>,
    request: PushToCommitRequest,
) -> AppResult<PushToCommitJobStarted> {
    validate_non_empty(&request.repo_path, "invalid_repository", "仓库路径不能为空")?;
    validate_non_empty(&request.branch, "invalid_branch", "目标分支不能为空")?;
    validate_non_empty(&request.hash, "invalid_hash", "目标 Commit 不能为空")?;

    let job_id = jobs.next_job_id.fetch_add(1, Ordering::SeqCst);
    let running_job = Arc::clone(&jobs.running_job);
    let task_key = format!("push-to-commit:{job_id}");
    let running_task = Arc::clone(&gate.running_task);

    reserve_push_execution(&running_task, &task_key)?;

    {
        let mut current = running_job.lock().expect("push to commit state poisoned");
        if current.is_some() {
            clear_push_execution(&running_task, &task_key);
            return Err(AppError::new(
                "push_to_commit_busy",
                "已有 push to commit 任务正在运行，请等待完成",
            ));
        }
        *current = Some(job_id);
    }

    let repo_path = request.repo_path.clone();
    let requested_branch = request.branch.clone();
    let hash = request.hash.clone();
    let event_target = hash.clone();
    let task_key_for_thread = task_key.clone();

    thread::spawn(move || {
        emit_push_to_commit_progress(
            &app,
            job_id,
            &event_target,
            PushTargetKind::Commit,
            "preparing",
        );

        let branch = match git::validate_current_branch(&repo_path, &requested_branch) {
            Ok(branch) => branch,
            Err(error) => {
                emit_push_to_commit_failed(
                    &app,
                    job_id,
                    &event_target,
                    PushTargetKind::Commit,
                    error,
                );
                clear_running_job(&running_job, job_id);
                clear_push_execution(&running_task, &task_key_for_thread);
                return;
            }
        };

        if let Err(error) = git::validate_push_target(&repo_path, &hash) {
            emit_push_to_commit_failed(&app, job_id, &event_target, PushTargetKind::Commit, error);
            clear_running_job(&running_job, job_id);
            clear_push_execution(&running_task, &task_key_for_thread);
            return;
        }

        emit_push_to_commit_progress(
            &app,
            job_id,
            &event_target,
            PushTargetKind::Commit,
            "running",
        );

        if let Err(error) = git::push_to_commit(&repo_path, &branch, &hash) {
            emit_push_to_commit_failed(&app, job_id, &event_target, PushTargetKind::Commit, error);
            clear_running_job(&running_job, job_id);
            clear_push_execution(&running_task, &task_key_for_thread);
            return;
        }

        let _ = app.emit(
            PUSH_TO_COMMIT_FINISHED_EVENT,
            PushToCommitFinished {
                job_id,
                target: event_target,
                target_kind: PushTargetKind::Commit,
            },
        );
        clear_running_job(&running_job, job_id);
        clear_push_execution(&running_task, &task_key_for_thread);
    });

    Ok(PushToCommitJobStarted {
        job_id,
        target: request.hash,
        target_kind: PushTargetKind::Commit,
    })
}

pub fn start_push_current_branch(
    app: AppHandle,
    jobs: State<'_, PushToCommitManager>,
    gate: State<'_, PushExecutionGate>,
    repo_path: String,
    branch: String,
) -> AppResult<PushToCommitJobStarted> {
    validate_non_empty(&repo_path, "invalid_repository", "仓库路径不能为空")?;
    validate_non_empty(&branch, "invalid_branch", "目标分支不能为空")?;

    let job_id = jobs.next_job_id.fetch_add(1, Ordering::SeqCst);
    let running_job = Arc::clone(&jobs.running_job);
    let task_key = format!("push-current-branch:{job_id}");
    let running_task = Arc::clone(&gate.running_task);

    reserve_push_execution(&running_task, &task_key)?;

    {
        let mut current = running_job.lock().expect("push state poisoned");
        if current.is_some() {
            clear_push_execution(&running_task, &task_key);
            return Err(AppError::new(
                "push_to_commit_busy",
                "已有 push 任务正在运行，请等待完成",
            ));
        }
        *current = Some(job_id);
    }

    let target = format!("{REMOTE_NAME}/{branch}");
    let target_for_thread = target.clone();
    let task_key_for_thread = task_key.clone();

    thread::spawn(move || {
        emit_push_to_commit_progress(
            &app,
            job_id,
            &target_for_thread,
            PushTargetKind::Branch,
            "preparing",
        );

        emit_push_to_commit_progress(
            &app,
            job_id,
            &target_for_thread,
            PushTargetKind::Branch,
            "running",
        );

        if let Err(error) = git::push_current_branch_checked(&repo_path, &branch) {
            emit_push_to_commit_failed(
                &app,
                job_id,
                &target_for_thread,
                PushTargetKind::Branch,
                error,
            );
            clear_running_job(&running_job, job_id);
            clear_push_execution(&running_task, &task_key_for_thread);
            return;
        }

        let _ = app.emit(
            PUSH_TO_COMMIT_FINISHED_EVENT,
            PushToCommitFinished {
                job_id,
                target: target_for_thread.clone(),
                target_kind: PushTargetKind::Branch,
            },
        );
        clear_running_job(&running_job, job_id);
        clear_push_execution(&running_task, &task_key_for_thread);
    });

    Ok(PushToCommitJobStarted {
        job_id,
        target,
        target_kind: PushTargetKind::Branch,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        clear_push_execution, clear_running_job, reserve_push_execution, validate_non_empty,
        validate_step_push_request_shape,
    };
    use crate::models::StepPushRequest;
    use std::sync::{Arc, Mutex};

    #[test]
    fn rejects_empty_push_request_fields() {
        let error = validate_non_empty("  ", "invalid_hash", "目标 Commit 不能为空").unwrap_err();

        assert_eq!(error.code, "invalid_hash");
        assert_eq!(error.message, "目标 Commit 不能为空");
    }

    #[test]
    fn rejects_invalid_step_push_request_shape() {
        let mut request = StepPushRequest {
            repo_path: "repo".to_string(),
            branch: "main".to_string(),
            hashes: vec!["abc123".to_string()],
            delay_ms: None,
        };

        request.repo_path = "  ".to_string();
        let error = validate_step_push_request_shape(&request).unwrap_err();
        assert_eq!(error.code, "invalid_repository");
        assert_eq!(error.message, "仓库路径不能为空");

        request.repo_path = "repo".to_string();
        request.branch = "  ".to_string();
        let error = validate_step_push_request_shape(&request).unwrap_err();
        assert_eq!(error.code, "invalid_branch");
        assert_eq!(error.message, "目标分支不能为空");

        request.branch = "main".to_string();
        request.hashes = Vec::new();
        let error = validate_step_push_request_shape(&request).unwrap_err();
        assert_eq!(error.code, "empty_hashes");
        assert_eq!(error.message, "没有可推送的 Commit");

        request.hashes = vec!["abc123".to_string(), "  ".to_string()];
        let error = validate_step_push_request_shape(&request).unwrap_err();
        assert_eq!(error.code, "invalid_hash");
        assert_eq!(error.message, "目标 Commit 不能为空");
    }

    #[test]
    fn rejects_parallel_push_execution_reservations() {
        let running_task = Arc::new(Mutex::new(None));

        reserve_push_execution(&running_task, "step-push:1").unwrap();

        let error = reserve_push_execution(&running_task, "push-to-commit:1").unwrap_err();
        assert_eq!(error.code, "push_busy");
        assert_eq!(error.message, "已有推送任务正在运行，请等待完成");
    }

    #[test]
    fn clears_only_matching_push_execution_reservation() {
        let running_task = Arc::new(Mutex::new(None));

        reserve_push_execution(&running_task, "step-push:1").unwrap();
        clear_push_execution(&running_task, "push-to-commit:1");
        assert_eq!(running_task.lock().unwrap().as_deref(), Some("step-push:1"));

        clear_push_execution(&running_task, "step-push:1");
        assert!(running_task.lock().unwrap().is_none());
    }

    #[test]
    fn clears_only_matching_running_job() {
        let running_job = Arc::new(Mutex::new(Some(7)));

        clear_running_job(&running_job, 8);
        assert_eq!(*running_job.lock().unwrap(), Some(7));

        clear_running_job(&running_job, 7);
        assert!(running_job.lock().unwrap().is_none());
    }
}
