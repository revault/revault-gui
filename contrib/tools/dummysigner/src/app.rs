use iced::{executor, Application, Clipboard, Column, Command, Container, Element, Settings};

pub fn run() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (400, 600);
    settings.window.resizable = false;
    App::run(settings)
}

pub struct App {}

#[derive(Debug)]
pub enum Message {}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (App {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Dummy signer - Revault")
    }

    fn update(&mut self, _message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Container::new(iced::Text::new("Hello"))
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
            .into()
    }
}
