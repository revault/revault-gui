use async_recursion::async_recursion;
use std::marker::PhantomData;

use revault_gui::{
    app::{context::Context, message::Message, state::State},
    daemon::client::Client,
};

pub struct Sandbox<C: Client + Send + Sync + 'static, S: State<C>> {
    state: S,
    marker: PhantomData<C>,
}

impl<C: Client + Send + Sync + 'static, S: State<C> + Send + 'static> Sandbox<C, S> {
    pub fn new(state: S) -> Self {
        return Self {
            state,
            marker: PhantomData,
        };
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    #[async_recursion]
    pub async fn update(mut self, ctx: &Context<C>, message: Message) -> Self {
        let cmd = self.state.update(ctx, message);
        for f in cmd.futures() {
            let msg = f.await;
            self = self.update(ctx, msg).await;
        }

        self
    }

    #[async_recursion]
    pub async fn load(mut self, ctx: &Context<C>) -> Self {
        let cmd = self.state.load(ctx);
        for f in cmd.futures() {
            let msg = f.await;
            self = self.update(ctx, msg).await;
        }

        self
    }
}
