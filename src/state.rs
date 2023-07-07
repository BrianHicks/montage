use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state")]
pub enum State {
    NothingIsHappening {},
    Running {
        task: String,
    },
    OnBreak {},
}
