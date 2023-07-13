#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct CurrentSessionQuery {
    pub current_session: Option<Session>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    pub end_time: Option<DateTime>,
    pub kind: Kind,
    pub projected_end_time: DateTime,
    pub remaining_time: Option<Duration>,
    pub start_time: DateTime,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum Kind {
    Task,
    Break,
}

type DateTime = chrono::DateTime<chrono::Local>;
cynic::impl_scalar!(DateTime, schema::DateTime);

type Duration = iso8601::Duration;
cynic::impl_scalar!(Duration, schema::Duration);

#[cfg(test)]
mod test {
    use super::*;
    use cynic::QueryBuilder;

    #[test]
    fn current_session_gql_output() {
        let operation = CurrentSessionQuery::build(());

        insta::assert_snapshot!(operation.query);
    }
}
