#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct ExtendByMutationVariables {
    pub duration: Duration,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ExtendByMutationVariables")]
pub struct ExtendByMutation {
    #[arguments(duration: $duration)]
    pub extend_by: Session,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub description: String,
    pub projected_end_time: DateTime,
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
    fn gql_output() {
        let operation = ExtendByMutation::build(ExtendByMutationVariables {
            duration: iso8601::duration("PT25M").unwrap(),
        });

        insta::assert_snapshot!(operation.query);
    }
}
