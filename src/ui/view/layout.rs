use crate::ui::{color, image::revault_colored_logo};
use iced::{container, Column, Container, Element, Length, Row};

pub fn cover<'a, T: 'a>(content: Container<'a, T>) -> Element<'a, T> {
    Column::new()
        .push(large_logo())
        .push(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .padding(50)
        .spacing(50)
        .align_items(iced::Align::Center)
        .into()
}

pub fn large_logo<T>() -> Container<'static, T> {
    Container::new(
        revault_colored_logo()
            .width(Length::Units(300))
            .height(Length::Fill),
    )
}

pub fn dashboard<'a, T: 'a>(
    header: Container<'a, T>,
    sidebar: Container<'a, T>,
    main: Container<'a, T>,
) -> Element<'a, T> {
    Column::new()
        .push(header)
        .push(
            Row::new()
                .push(sidebar.width(Length::Shrink).height(Length::Fill))
                .push(main.width(Length::Fill).height(Length::Fill)),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

pub fn sidebar<'a, T: 'a>(menu: Container<'a, T>, footer: Container<'a, T>) -> Container<'a, T> {
    Container::new(
        Column::new()
            .padding(20)
            .push(menu.height(Length::Fill))
            .push(footer.height(Length::Shrink)),
    )
    .style(SidebarStyle)
}

pub struct SidebarStyle;
impl container::StyleSheet for SidebarStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::BACKGROUND.into(),
            ..container::Style::default()
        }
    }
}

pub fn sidebar_menu<'a, T: 'a>(items: Vec<Container<'a, T>>) -> Container<'a, T> {
    let mut col = Column::new().padding(20).spacing(10);
    for i in items {
        col = col.push(i)
    }
    Container::new(col).style(MainSectionStyle)
}

pub fn main_section<'a, T: 'a>(menu: Container<'a, T>) -> Container<'a, T> {
    Container::new(menu).padding(20).style(MainSectionStyle)
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
