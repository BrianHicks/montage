use color_eyre::eyre::{bail, Result, WrapErr};
use std::path::PathBuf;
use std::process::Command;

pub enum Script {
    SessionStarted,
}

impl Script {
    fn filename(&self) -> &'static str {
        match self {
            Self::SessionStarted => "session_started",
        }
    }

    fn args(&self) -> &[&str] {
        match self {
            Self::SessionStarted => &[],
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
