use chrono::Duration;
use color_eyre::eyre::{bail, Result, WrapErr};
use montage_client::current_session_updates::Session;
use std::path::PathBuf;
use std::process::Command;

pub enum Script<'arg> {
    NewSession {
        session: &'arg Session,
    },
    SessionEnded {
        session: &'arg Session,
    },
    Reminder {
        session: &'arg Session,
        reminder: &'arg Duration,
    },
    SessionOverTime {
        session: &'arg Session,
    },
}

impl Script<'_> {
    fn filename(&self) -> &'static str {
        match self {
            Self::NewSession { .. } => "new_session",
            Self::SessionEnded { .. } => "session_ended",
            Self::Reminder { .. } => "reminder",
            Self::SessionOverTime { .. } => "session_over_time",
        }
    }

    fn session(&self) -> &Session {
        match self {
            Self::NewSession { session } => session,
            Self::SessionEnded { session } => session,
            Self::Reminder { session, .. } => session,
            Self::SessionOverTime { session } => session,
        }
    }

    fn args(&self) -> Vec<String> {
        match self {
            Self::NewSession { session } => {
                vec![
                    session.description.clone(),
                    session.kind.to_string(),
                    session.projected_end_time.to_string(),
                ]
            }
            Self::SessionEnded { session } => vec![session.description.clone()],
            Self::Reminder { session, reminder } => vec![
                session.description.clone(),
                reminder.num_seconds().to_string(),
            ],
            Self::SessionOverTime { session } => vec![session.description.clone()],
        }
    }

    fn env(&self) -> Vec<(&str, String)> {
        vec![("SESSION", serde_json::to_string(self.session()).unwrap())]
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
                .envs(self.env())
                .status()
                .wrap_err_with(|| format!("failed to run {script_name}"))?;

            if !status.success() {
                bail!("`{script_name}` failed with status {status}")
            }
        }

        Ok(())
    }
}
