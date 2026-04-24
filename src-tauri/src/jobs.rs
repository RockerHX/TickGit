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
        StepPushFailed, StepPushFinished, StepPushJobStarted, StepPushProgress, StepPushRequest,
    },
};

pub const STEP_PUSH_PROGRESS_EVENT: &str = "step-push-progress";
pub const STEP_PUSH_FINISHED_EVENT: &str = "step-push-finished";
pub const STEP_PUSH_FAILED_EVENT: &str = "step-push-failed";

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

fn clear_running_job(running_job: &Arc<Mutex<Option<u64>>>, job_id: u64) {
    if let Ok(mut current) = running_job.lock() {
        if current.as_ref() == Some(&job_id) {
            *current = None;
        }
    }
}

pub fn start_step_push(
    app: AppHandle,
    jobs: State<'_, StepPushManager>,
    request: StepPushRequest,
) -> AppResult<StepPushJobStarted> {
    git::validate_repository(&request.repo_path)?;

    if request.branch.trim().is_empty() {
        return Err(AppError::new("invalid_branch", "目标分支不能为空"));
    }

    if request.hashes.is_empty() {
        return Err(AppError::new("empty_hashes", "没有可推送的 Commit"));
    }

    let job_id = jobs.next_job_id.fetch_add(1, Ordering::SeqCst);
    let running_job = Arc::clone(&jobs.running_job);

    {
        let mut current = running_job.lock().expect("step push state poisoned");
        if current.is_some() {
            return Err(AppError::new(
                "step_push_busy",
                "已有分步提交任务正在运行，请等待完成",
            ));
        }
        *current = Some(job_id);
    }

    let total = request.hashes.len();
    let repo_path = request.repo_path.clone();
    let branch = request.branch.clone();
    let hashes = request.hashes.clone();
    let delay_ms = request.delay_ms.unwrap_or(1500);

    thread::spawn(move || {
        for (index, hash) in hashes.iter().enumerate() {
            if let Err(error) = git::push_to_commit(&repo_path, &branch, hash) {
                let _ = app.emit(
                    STEP_PUSH_FAILED_EVENT,
                    StepPushFailed {
                        job_id,
                        current: index + 1,
                        total,
                        hash: hash.clone(),
                        message: error.message,
                    },
                );
                clear_running_job(&running_job, job_id);
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
    });

    Ok(StepPushJobStarted { job_id, total })
}
