pub mod syncing {
    use iced::{Align, Column, Container, Element, Length, ProgressBar, Text};
    use iced_futures::futures;

    /// SyncingProgress specifies the application syncing progress.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Progress {
        Pending(f32),
        Finished,
        Errored,
    }

    pub enum State {
        Unknown,
        Pending { progress: f32 },
        Synced,
    }

    pub fn view(progress: &Progress) -> Element<Progress> {
        match progress {
            Progress::Pending(p) => {
                let progress_bar = ProgressBar::new(0.0..=100.0, *p);
                let content = Column::new()
                    .spacing(10)
                    .padding(10)
                    .align_items(Align::Center)
                    .push(progress_bar);
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            Progress::Finished => Text::new("Application starting...").into(),
            Progress::Errored => Text::new("An error occured...").into(),
        }
    }

    pub struct SyncObserver {}

    impl SyncObserver {
        pub fn new() -> SyncObserver {
            SyncObserver {}
        }
    }

    impl<H, I> iced_native::subscription::Recipe<H, I> for SyncObserver
    where
        H: std::hash::Hasher,
    {
        type Output = Progress;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            Box::pin(futures::stream::unfold(
                State::Unknown,
                |state| async move {
                    match state {
                        State::Unknown => {
                            Some((Progress::Pending(00.0), State::Pending { progress: 00.0 }))
                        }
                        State::Pending { mut progress } => {
                            if progress > 100.0 as f32 {
                                return Some((Progress::Finished, State::Synced));
                            }
                            std::thread::sleep(std::time::Duration::from_millis(2));
                            progress += 0.1;
                            Some((Progress::Pending(progress), State::Pending { progress }))
                        }
                        State::Synced => {
                            // We do not let the stream die, as it would start a
                            // new download repeatedly if the user is not careful
                            // in case of errors.
                            let _: () = iced::futures::future::pending().await;

                            None
                        }
                    }
                },
            ))
        }
    }
}
