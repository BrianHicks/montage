use chrono::Duration;
use color_eyre::eyre::{bail, Result, WrapErr};
use std::path::PathBuf;
use std::process::Command;

pub enum Script<'arg> {
    SessionStarted,
    Reminder { reminder: &'arg Duration },
}

impl Script<'_> {
    fn filename(&self) -> &'static str {
        match self {
            Self::SessionStarted => "session_started",
            Self::Reminder { .. } => "reminder",
        }
    }

    fn args(&self) -> Vec<String> {
        match self {
            Self::SessionStarted => Vec::new(),
            Self::Reminder { reminder } => vec![reminder.num_seconds().to_string()],
        }
    }

    pub fn run_from(&self, script_dir: &Option<PathBuf>) -> Result<()> {
        if let Some(script_dir) = script_dir {
            let script_name = self.filename();
            let script_path = script_dir.join(script_name);

            if !script_path.exists() {
                tracing::debug!(
                    script_name = script_name,
                    "wanted to run script but it doesn't exist"
                );
                return Ok(());
            }

            tracing::info!(script_name = script_name, "running script");
            let status = Command::new(script_path)
                .args(self.args())
                .status()
                .wrap_err_with(|| format!("failed to run {script_name}"))?;

            if !status.success() {
                bail!("`{script_name}` failed with status {status}")
            }
        }

        Ok(())
    }
}
