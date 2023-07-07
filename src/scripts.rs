use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::{Help, SectionExt};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Scripts(PathBuf);

impl std::ops::Deref for Scripts {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn value_parser(s: &str) -> Result<Scripts> {
    Ok(Scripts(PathBuf::from(s)))
}

impl Scripts {
    pub fn on_start(&self, name: &str) -> Result<()> {
        let to_run = self.join("on-start");
        if to_run.exists() {
            let mut command = Command::new(to_run);
            command.arg(name);

            Scripts::run("start", command)?;
        }

        Ok(())
    }

    fn run(name: &str, mut command: Command) -> Result<()> {
        let output = command
            .output()
            .wrap_err_with(|| format!("{} script failed to start", name))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(eyre!("{} script failed", name)
                .section(stdout.trim().to_string().header("Stdout:"))
                .section(stderr.trim().to_string().header("Stderr:")));
        }

        Ok(())
    }
}
