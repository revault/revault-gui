use iced_native::command::Action;
use revault_gui::app::{context::Context, message::Message, state::State};

pub struct Sandbox<S: State> {
    state: S,
}

impl<S: State + 'static> Sandbox<S> {
    pub fn new(state: S) -> Self {
        return Self { state };
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub async fn update(mut self, ctx: &Context, message: Message) -> Self {
        let cmd = self.state.update(ctx, message);
        for action in cmd.actions() {
            if let Action::Future(f) = action {
                let msg = f.await;
                let _cmd = self.state.update(ctx, msg);
            }
        }

        self
    }

    pub async fn load(mut self, ctx: &Context) -> Self {
        let cmd = self.state.load(ctx);
        for action in cmd.actions() {
            if let Action::Future(f) = action {
                let msg = f.await;
                self = self.update(ctx, msg).await;
            }
        }

        self
    }
}
