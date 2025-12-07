use std::sync::Arc;
use tokio::time::{interval, Duration};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Scheduler {
    running: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self, schedule: &str) {
        // For MVP, we'll use a simple interval-based scheduler
        // In production, you'd want to use a cron parser like 'cron' or 'tokio-cron-scheduler'

        self.running.store(true, Ordering::SeqCst);

        // Parse schedule string for simple interval (e.g., "daily", "hourly")
        let duration = match schedule {
            "hourly" => Duration::from_secs(3600),
            "daily" => Duration::from_secs(86400),
            _ => Duration::from_secs(86400), // Default to daily
        };

        let running = self.running.clone();
        let mut interval_timer = interval(duration);

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                interval_timer.tick().await;

                // Trigger research
                if let Err(e) = Self::run_research().await {
                    eprintln!("Scheduled research failed: {}", e);
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn run_research() -> Result<(), String> {
        use tokio::process::Command;

        let output = Command::new("claudius")
            .arg("research")
            .output()
            .await
            .map_err(|e| format!("Failed to execute claudius CLI: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Research failed: {}", stderr))
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
