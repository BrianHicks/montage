use super::session::Session;
use async_graphql::Subscription;
use futures::stream::Stream;
use tokio::sync::watch::Receiver;
use tokio_stream::wrappers::WatchStream;

pub struct Subscription {
    receiver: Receiver<Option<Session>>,
}

impl Subscription {
    pub fn new(receiver: Receiver<Option<Session>>) -> Self {
        Self { receiver }
    }
}

#[Subscription]
impl Subscription {
    /// Get the current session and any future sessions while the connection is open.
    async fn current_session(&self) -> impl Stream<Item = Option<Session>> {
        WatchStream::new(self.receiver.clone())
    }
}
