use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, error, warn};
use cron::Schedule;
use std::str::FromStr;
use chrono::Local;
use tauri::AppHandle;

/// Research scheduler that runs research at configured times using cron expressions.
pub struct Scheduler {
    scheduler: Arc<RwLock<Option<JobScheduler>>>,
    current_schedule: Arc<RwLock<Option<String>>>,
    app_handle: Arc<RwLock<Option<AppHandle>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            scheduler: Arc::new(RwLock::new(None)),
            current_schedule: Arc::new(RwLock::new(None)),
            app_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the app handle for notifications.
    pub async fn set_app_handle(&self, handle: AppHandle) {
        let mut app_handle_lock = self.app_handle.write().await;
        *app_handle_lock = Some(handle);
    }

    /// Get a clone of the app handle.
    #[allow(dead_code)]
    pub async fn get_app_handle(&self) -> Option<AppHandle> {
        let app_handle_lock = self.app_handle.read().await;
        app_handle_lock.clone()
    }

    /// Start the scheduler with the given cron expression.
    ///
    /// # Arguments
    /// * `cron_expr` - A cron expression like "0 6 * * *" (6 AM daily)
    ///
    /// # Cron Format
    /// ```
    /// ┌───────────── second (0 - 59) [optional]
    /// │ ┌───────────── minute (0 - 59)
    /// │ │ ┌───────────── hour (0 - 23)
    /// │ │ │ ┌───────────── day of month (1 - 31)
    /// │ │ │ │ ┌───────────── month (1 - 12)
    /// │ │ │ │ │ ┌───────────── day of week (0 - 6) (Sunday = 0)
    /// │ │ │ │ │ │
    /// * * * * * *
    /// ```
    pub async fn start(&self, cron_expr: &str) -> Result<(), String> {
        // Stop existing scheduler if running
        self.stop().await;

        // Validate and normalize the cron expression
        // If it's a 5-part expression (standard cron), add seconds
        let full_cron = if cron_expr.split_whitespace().count() == 5 {
            format!("0 {}", cron_expr) // Add "0" for seconds
        } else {
            cron_expr.to_string()
        };

        info!("Starting scheduler with cron: {}", full_cron);

        // Get a clone of the app_handle for the job
        let app_handle_arc = self.app_handle.clone();

        // Create the scheduler
        let sched = JobScheduler::new().await
            .map_err(|e| format!("Failed to create scheduler: {}", e))?;

        // Create the job
        let job = Job::new_async(full_cron.as_str(), move |_uuid, _lock| {
            let app_handle_arc = app_handle_arc.clone();
            Box::pin(async move {
                info!("Scheduled research triggered");
                let app_handle = {
                    let lock = app_handle_arc.read().await;
                    lock.clone()
                };
                if let Err(e) = run_research(app_handle).await {
                    error!("Scheduled research failed: {}", e);
                } else {
                    info!("Scheduled research completed successfully");
                }
            })
        }).map_err(|e| format!("Failed to create job: {}", e))?;

        // Add job to scheduler
        sched.add(job).await
            .map_err(|e| format!("Failed to add job: {}", e))?;

        // Start the scheduler
        sched.start().await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;

        // Store the scheduler and schedule
        {
            let mut sched_lock = self.scheduler.write().await;
            *sched_lock = Some(sched);
        }
        {
            let mut schedule_lock = self.current_schedule.write().await;
            *schedule_lock = Some(cron_expr.to_string());
        }

        info!("Scheduler started successfully");
        Ok(())
    }

    /// Stop the scheduler.
    pub async fn stop(&self) {
        let mut sched_lock = self.scheduler.write().await;
        if let Some(mut sched) = sched_lock.take() {
            if let Err(e) = sched.shutdown().await {
                warn!("Error shutting down scheduler: {}", e);
            } else {
                info!("Scheduler stopped");
            }
        }

        let mut schedule_lock = self.current_schedule.write().await;
        *schedule_lock = None;
    }

    /// Update the schedule with a new cron expression.
    #[allow(dead_code)]
    pub async fn update_schedule(&self, cron_expr: &str) -> Result<(), String> {
        let current = self.current_schedule.read().await;
        if current.as_deref() == Some(cron_expr) {
            // Schedule hasn't changed
            return Ok(());
        }
        drop(current);

        self.start(cron_expr).await
    }

    /// Check if the scheduler is running.
    pub async fn is_running(&self) -> bool {
        let sched_lock = self.scheduler.read().await;
        sched_lock.is_some()
    }

    /// Get the current schedule.
    pub async fn get_schedule(&self) -> Option<String> {
        let schedule_lock = self.current_schedule.read().await;
        schedule_lock.clone()
    }

    /// Get the next scheduled run time.
    pub async fn get_next_run_time(&self) -> Option<String> {
        let schedule_lock = self.current_schedule.read().await;
        if let Some(cron_str) = schedule_lock.as_ref() {
            // Normalize to 6-part cron (with seconds)
            let full_cron = if cron_str.split_whitespace().count() == 5 {
                format!("0 {}", cron_str)
            } else {
                cron_str.clone()
            };

            if let Ok(schedule) = Schedule::from_str(&full_cron) {
                if let Some(next) = schedule.upcoming(Local).next() {
                    return Some(next.format("%H:%M").to_string());
                }
            }
        }
        None
    }

    /// Get the next scheduled run time with full datetime.
    #[allow(dead_code)]
    pub async fn get_next_run_datetime(&self) -> Option<chrono::DateTime<Local>> {
        let schedule_lock = self.current_schedule.read().await;
        if let Some(cron_str) = schedule_lock.as_ref() {
            let full_cron = if cron_str.split_whitespace().count() == 5 {
                format!("0 {}", cron_str)
            } else {
                cron_str.clone()
            };

            if let Ok(schedule) = Schedule::from_str(&full_cron) {
                return schedule.upcoming(Local).next();
            }
        }
        None
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the research agent using the Rust research module.
async fn run_research(app_handle: Option<AppHandle>) -> Result<(), String> {
    info!("Executing research via Rust agent...");

    // Call the trigger_research command which uses the Rust research agent
    // If we have an app handle, use the command that supports notifications
    match app_handle {
        Some(handle) => {
            match crate::commands::trigger_research(handle).await {
                Ok(msg) => {
                    info!("Scheduled research completed: {}", msg);
                    Ok(())
                }
                Err(e) => {
                    error!("Scheduled research failed: {}", e);
                    Err(e)
                }
            }
        }
        None => {
            // No app handle available, run without notifications
            warn!("Running research without app handle - notifications disabled");
            match crate::commands::trigger_research_no_notify().await {
                Ok(msg) => {
                    info!("Scheduled research completed: {}", msg);
                    Ok(())
                }
                Err(e) => {
                    error!("Scheduled research failed: {}", e);
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        let scheduler = Scheduler::new();

        assert!(!scheduler.is_running().await);
        assert!(scheduler.get_schedule().await.is_none());

        // Note: This test would need a valid cron expression
        // For unit testing, we'd mock the actual job execution
    }
}
