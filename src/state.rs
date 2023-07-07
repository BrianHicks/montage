use chrono::{DateTime, Local};
use color_eyre::eyre::{eyre, Result, WrapErr};
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

impl State {
    pub fn start(&self, task: String, until: DateTime<Local>) -> State {
        State::Running { task, until }
    }

    pub fn start_break(&self, until: DateTime<Local>) -> State {
        State::OnBreak { until }
    }

    pub fn stop(&self) -> State {
        State::NothingIsHappening {}
    }
}

pub struct Store {
    loaded_from: PathBuf,
    pub state: State,

    // watching
    watcher: Option<(
        notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
        crossbeam_channel::Receiver<()>,
    )>,
}

/// Note for the future: the loads and writes aren't doing any kind of locking on any platform.
/// This probably won't be a huge problem, but it might be eventually. It's something to watch out
/// for, certainly!
impl Store {
    pub fn create_or_load() -> Result<Self> {
        let state_file = directories::ProjectDirs::from("zone", "bytes", "montage")
            .ok_or(eyre!("could not determine config location"))?
            .data_local_dir()
            .join("state.json");

        if !state_file.exists() {
            let store = Store {
                loaded_from: state_file,
                state: State::default(),
                watcher: None,
            };

            store.write().wrap_err("could not write initial state")?;

            Ok(store)
        } else {
            let state_bytes =
                std::fs::read_to_string(&state_file).wrap_err("could not read state file")?;
            let state =
                serde_json::from_str(&state_bytes).wrap_err("could not deserialize state")?;

            Ok(Store {
                loaded_from: state_file,
                state,
                watcher: None,
            })
        }
    }

    pub fn write(&self) -> Result<()> {
        if let Some(parent) = self.loaded_from.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .wrap_err("could not create parent directories to store")?;
            }
        }

        let mut file = File::create(&self.loaded_from).wrap_err("could not open state file")?;
        write!(
            file,
            "{}",
            serde_json::to_string(&self.state).wrap_err("could not serialize state")?
        )
        .wrap_err("failed to write state file to disk")?;

        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        let state_bytes =
            std::fs::read_to_string(&self.loaded_from).wrap_err("could not read state file")?;
        self.state =
            serde_json::from_str(&state_bytes).wrap_err("could not deserialize state")?;

        Ok(())
    }

    pub fn watch(&mut self) -> Result<crossbeam_channel::Receiver<()>> {
        match &self.watcher {
            Some((_, recv)) => Ok(recv.clone()),
            None => {
                let (send, recv) = crossbeam_channel::bounded(1);

                let mut debouncer = notify_debouncer_mini::new_debouncer(
                    std::time::Duration::from_millis(500),
                    None,
                    move |event_res| match event_res {
                        Ok(_) => send.send(()).unwrap(),
                        Err(err) => eprintln!("error in watcher: {:?}", err),
                    },
                )
                .wrap_err("could not start file watcher")?;

                debouncer
                    .watcher()
                    .watch(&self.loaded_from, notify::RecursiveMode::NonRecursive)
                    .wrap_err("could not watch store path")?;

                let to_return = recv.clone();
                self.watcher = Some((debouncer, recv));

                Ok(to_return)
            }
        }
    }

    pub fn start(&mut self, task: String, until: DateTime<Local>) {
        self.state = self.state.start(task, until);
    }

    pub fn start_break(&mut self, until: DateTime<Local>) {
        self.state = self.state.start_break(until);
    }

    pub fn stop(&mut self) {
        self.state = self.state.stop();
    }
}
