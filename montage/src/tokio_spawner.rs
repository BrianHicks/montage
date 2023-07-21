// NOTE: I got this code from
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c63e153f8a0eae7af6f84e7a7f76fb73,
// which is linked to from https://github.com/tokio-rs/tokio/issues/4297. I'm not sure what the
// intended license is! Use at your own risk!

pub struct TokioSpawner(tokio::runtime::Handle);

impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioSpawner(handle)
    }

    pub fn current() -> Self {
        TokioSpawner::new(tokio::runtime::Handle::current())
    }
}

impl futures::task::Spawn for TokioSpawner {
    fn spawn_obj(
        &self,
        obj: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.0.spawn(obj);
        Ok(())
    }
}
