/// What kind of session are we going to have?
#[derive(async_graphql::Enum, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    /// A longer session focused on doing something
    Task,

    /// A shorter recovery session
    Break,
}
