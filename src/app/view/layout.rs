use revault_ui::{
    color,
    component::{card, navbar, scroll, text::Text},
};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::{sidebar::Sidebar, warning::warn},
    },
    daemon::client::Client,
};

use iced::{container, scrollable, Column, Container, Element, Length, Row};

pub fn navbar_warning<'a, T: 'a>(warning: Option<&Error>) -> Option<Container<'a, T>> {
    if let Some(e) = warning {
        return Some(card::alert_warning(Container::new(Text::new(&format!(
            "{}",
            e
        )))));
    }
    None
}

#[derive(Debug, Clone)]
pub struct Dashboard {
    sidebar: Sidebar,
    scroll: scrollable::State,
}

impl Dashboard {
    pub fn new() -> Dashboard {
        Self {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a, C: Client, T: Into<Element<'a, Message>>>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        content: T,
    ) -> Element<'a, Message> {
        Column::new()
            .push(navbar())
            .push(
                Row::new()
                    .push(
                        self.sidebar
                            .view(ctx)
                            .width(Length::Shrink)
                            .height(Length::Fill),
                    )
                    .push(
                        Column::new().push(warn(warning)).push(
                            main_section(Container::new(scroll(
                                &mut self.scroll,
                                Container::new(content),
                            )))
                            .width(Length::Fill)
                            .height(Length::Fill),
                        ),
                    ),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

pub fn main_section<'a, T: 'a>(menu: Container<'a, T>) -> Container<'a, T> {
    Container::new(menu.max_width(1500))
        .padding(20)
        .style(MainSectionStyle)
        .align_x(iced::Align::Center)
        .width(Length::Fill)
        .height(Length::Fill)
}

pub struct MainSectionStyle;
impl container::StyleSheet for MainSectionStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::BACKGROUND.into(),
            ..container::Style::default()
        }
    }
}
