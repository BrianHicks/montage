#[cynic::schema("montage")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct ExtendToMutationVariables {
    pub target: DateTime,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "ExtendToMutationVariables")]
pub struct ExtendToMutation {
    #[arguments(target: $target)]
    pub extend_to: Session,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Session {
    pub projected_end_time: DateTime,
    pub description: String,
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
        let operation = ExtendToMutation::build(ExtendToMutationVariables {
            target: chrono::Local::now(),
        });

        insta::assert_snapshot!(operation.query);
    }
}
