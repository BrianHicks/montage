#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct StartMutationVariables<'a> {
    pub description: &'a str,
    pub kind: Kind,
    pub duration: Option<Duration>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "StartMutationVariables")]
pub struct StartMutation {
    #[arguments(description: $description, kind: $kind, duration: $duration)]
    pub start: Session,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub projected_end_time: DateTime,
    pub duration: Duration,
    pub description: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum Kind {
    Task,
    Break,
    NotWorking,
}

type DateTime = chrono::DateTime<chrono::Local>;
cynic::impl_scalar!(DateTime, schema::DateTime);

type Duration = iso8601::Duration;
cynic::impl_scalar!(Duration, schema::Duration);

#[cfg(test)]
mod test {
    use super::*;
    use cynic::MutationBuilder;

    #[test]
    fn start_gql_output() {
        let operation = StartMutation::build(StartMutationVariables {
            description: "test description",
            kind: Kind::Task,
            duration: None,
        });

        insta::assert_snapshot!(operation.query);
    }
}
