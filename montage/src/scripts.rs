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
    Annoy {
        session: &'arg Session,
    },
}

impl Script<'_> {
    fn filename(&self) -> &'static str {
        match self {
            Self::NewSession { .. } => "new_session",
            Self::SessionEnded { .. } => "session_ended",
            Self::Reminder { .. } => "reminder",
            Self::Annoy { .. } => "annoy",
        }
    }

    fn session(&self) -> &Session {
        match self {
            Self::NewSession { session } => session,
            Self::SessionEnded { session } => session,
            Self::Reminder { session, .. } => session,
            Self::Annoy { session } => session,
        }
    }

    fn args(&self) -> Vec<String> {
        match self {
            Self::NewSession { session } => {
                vec![
                    session.description.clone(),
                    session.kind.to_string(),
                    session.duration.to_string(),
                ]
            }
            Self::SessionEnded { .. } => vec![],
            Self::Reminder { reminder, .. } => vec![reminder.num_seconds().to_string()],
            Self::Annoy { .. } => vec![],
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
