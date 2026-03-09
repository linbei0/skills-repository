use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::domain::types::{TaskHandle, TaskProgressEvent};

pub const TASK_PROGRESS_EVENT: &str = "task:progress";
pub const TASK_COMPLETED_EVENT: &str = "task:completed";
pub const TASK_FAILED_EVENT: &str = "task:failed";

pub fn new_task_handle(task_type: &str) -> TaskHandle {
    TaskHandle {
        task_id: Uuid::new_v4().to_string(),
        task_type: task_type.to_string(),
    }
}

pub fn emit_progress(
    app: &AppHandle,
    task: &TaskHandle,
    status: &str,
    step: &str,
    current: u32,
    total: u32,
    message: &str,
) -> Result<()> {
    app.emit(
        TASK_PROGRESS_EVENT,
        TaskProgressEvent {
            task_id: task.task_id.clone(),
            task_type: task.task_type.clone(),
            status: status.to_string(),
            step: step.to_string(),
            current,
            total,
            message: message.to_string(),
            payload: None,
        },
    )
    .context("failed to emit task progress event")
}

pub fn emit_completed<T: Serialize>(
    app: &AppHandle,
    task: &TaskHandle,
    step: &str,
    message: &str,
    payload: T,
) -> Result<()> {
    let payload = serde_json::to_value(payload).context("failed to serialize task payload")?;

    app.emit(
        TASK_COMPLETED_EVENT,
        TaskProgressEvent {
            task_id: task.task_id.clone(),
            task_type: task.task_type.clone(),
            status: "completed".to_string(),
            step: step.to_string(),
            current: 1,
            total: 1,
            message: message.to_string(),
            payload: Some(payload),
        },
    )
    .context("failed to emit task completed event")
}

pub fn emit_partial<T: Serialize>(
    app: &AppHandle,
    task: &TaskHandle,
    step: &str,
    message: &str,
    payload: T,
) -> Result<()> {
    let payload = serde_json::to_value(payload).context("failed to serialize task payload")?;

    app.emit(
        TASK_COMPLETED_EVENT,
        TaskProgressEvent {
            task_id: task.task_id.clone(),
            task_type: task.task_type.clone(),
            status: "partial".to_string(),
            step: step.to_string(),
            current: 1,
            total: 1,
            message: message.to_string(),
            payload: Some(payload),
        },
    )
    .context("failed to emit task partial event")
}

pub fn emit_failed(app: &AppHandle, task: &TaskHandle, step: &str, message: &str) -> Result<()> {
    app.emit(
        TASK_FAILED_EVENT,
        TaskProgressEvent {
            task_id: task.task_id.clone(),
            task_type: task.task_type.clone(),
            status: "failed".to_string(),
            step: step.to_string(),
            current: 1,
            total: 1,
            message: message.to_string(),
            payload: None,
        },
    )
    .context("failed to emit task failed event")
}

pub fn emit_failed_with_payload<T: Serialize>(
    app: &AppHandle,
    task: &TaskHandle,
    step: &str,
    message: &str,
    payload: T,
) -> Result<()> {
    let payload =
        serde_json::to_value(payload).context("failed to serialize task failed payload")?;

    app.emit(
        TASK_FAILED_EVENT,
        TaskProgressEvent {
            task_id: task.task_id.clone(),
            task_type: task.task_type.clone(),
            status: "failed".to_string(),
            step: step.to_string(),
            current: 1,
            total: 1,
            message: message.to_string(),
            payload: Some(payload),
        },
    )
    .context("failed to emit task failed event")
}
