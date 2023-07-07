use chrono::{DateTime, Local};
use serde;

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
