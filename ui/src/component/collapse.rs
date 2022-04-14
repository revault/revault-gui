use iced::button::State;
use iced_lazy::Component;

pub struct Collapse<'a> {
    button: &'a mut State,
    collapsed: bool,
}
