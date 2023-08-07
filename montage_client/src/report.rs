use serde::Serialize;

#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct ReportQueryVariables {
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ReportQueryVariables")]
pub struct ReportQuery {
    #[arguments(end: $end, start: $start)]
    pub report: Report,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Report {
    pub start: DateTime,
    pub end: DateTime,
    pub totals: Totals,
    pub sessions: Vec<Session>,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Totals {
    pub short_break: Duration,
    pub long_break: Duration,
    pub task: Duration,
    pub meeting: Duration,
    pub working: Duration,
    pub sessions_by_description: Vec<TotalByDescription>,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct TotalByDescription {
    pub description: String,
    pub kind: Kind,
    pub total: Duration,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Session {
    pub description: String,
    pub actual_duration: Duration,
    pub kind: Kind,
    pub start_time: DateTime,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum Kind {
    Task,
    Break,
    Meeting,
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
    fn gql_output() {
        let operation = ReportQuery::build(ReportQueryVariables {
            start: chrono::Local::now(),
            end: chrono::Local::now(),
        });

        insta::assert_snapshot!(operation.query);
    }
}
