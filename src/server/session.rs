use async_graphql::SimpleObject;

#[derive(SimpleObject, Debug)]
pub struct Session {
    pub id: i64,
    pub kind: String,
    pub description: String,
    pub start_time: chrono::DateTime<chrono::Local>,
    pub end_time: chrono::DateTime<chrono::Local>,
}
