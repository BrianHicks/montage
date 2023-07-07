use chrono::{DateTime, Local};
use color_eyre::eyre::{bail, eyre, Result, WrapErr};
use serde;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state")]
pub enum State {
    NothingIsHappening {},
    Running {
        task: String,
        until: DateTime<Local>,
    },
    OnBreak {
        until: DateTime<Local>,
    },
}

impl Default for State {
    fn default() -> Self {
        State::NothingIsHappening {}
    }
}

#[derive(Debug)]
pub struct Store {
    loaded_from: Option<PathBuf>,
    state: State,
}

impl Store {
    pub fn create_or_load() -> Result<Self> {
        let state_file = directories::ProjectDirs::from("zone", "bytes", "montage")
            .ok_or(eyre!("could not determine config location"))?
            .data_local_dir()
            .join("state.json");

        if !state_file.exists() {
            let store = Store {
                loaded_from: Some(state_file),
                state: State::default(),
            };

            store.write().wrap_err("could not write initial state")?;

            Ok(store)
        } else {
            bail!("TODO")
        }
    }

    pub fn write(&self) -> Result<()> {
        match &self.loaded_from {
            None => bail!("this store wasn't loaded from disk, so it can't be written back"),
            Some(loaded_from) => {
                if let Some(parent) = loaded_from.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)
                            .wrap_err("could not create parent directories to store")?;
                    }
                }

                let mut file = File::create(&loaded_from).wrap_err("could not open state file")?;
                write!(
                    file,
                    "{}",
                    serde_json::to_string(&self.state).wrap_err("could not serialize state")?
                )
                .wrap_err("failed to write state file to disk")?;

                Ok(())
            }
        }
    }
}
