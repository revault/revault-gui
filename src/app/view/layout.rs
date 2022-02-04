use revault_ui::{
    color,
    component::{button, navbar, scroll, text::Text, ContainerBackgroundStyle, TooltipStyle},
    icon,
};

use crate::app::{
    context::Context,
    error::Error,
    message::Message,
    view::{sidebar::Sidebar, warning::warn},
};

use iced::{
    container, scrollable, tooltip, Alignment, Column, Container, Element, Length, Row, Tooltip,
};

#[derive(Debug, Clone, Default)]
pub struct Dashboard {
    sidebar: Sidebar,
    scroll: scrollable::State,
}

impl Dashboard {
    pub fn new() -> Dashboard {
        Self {
            sidebar: Sidebar::default(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a, T: Into<Element<'a, Message>>>(
        &'a mut self,
        ctx: &Context,
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
        .center_x()
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

#[derive(Debug, Default)]
pub struct Modal {
    scroll: scrollable::State,
    close_button: iced::button::State,
}

impl Modal {
    pub fn new() -> Self {
        Modal {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, T: Into<Element<'a, Message>>>(
        &'a mut self,
        _ctx: &Context,
        warning: Option<&Error>,
        content: T,
        tooltip: Option<&str>,
        close_redirect: Message,
    ) -> Element<'a, Message> {
        let tt = if let Some(help) = tooltip {
            Container::new(
                Tooltip::new(
                    Row::new()
                        .push(icon::tooltip_icon().size(20))
                        .push(Text::new(" Help")),
                    help,
                    tooltip::Position::Right,
                )
                .gap(5)
                .size(20)
                .padding(10)
                .style(TooltipStyle),
            )
        } else {
            Container::new(Column::new())
        };
        let col = Column::new()
            .push(
                Column::new()
                    .push(warn(warning))
                    .push(
                        Row::new()
                            .push(tt.width(Length::Fill))
                            .push(
                                Container::new(
                                    button::close_button(&mut self.close_button)
                                        .on_press(close_redirect),
                                )
                                .width(Length::Shrink),
                            )
                            .align_items(Alignment::Center)
                            .padding(20),
                    )
                    .spacing(20),
            )
            .push(
                Container::new(Container::new(content).max_width(1500))
                    .width(Length::Fill)
                    .center_x(),
            )
            .spacing(50);

        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .into()
    }
}
