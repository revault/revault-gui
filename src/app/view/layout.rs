use revault_ui::{
    color,
    component::{card, text::Text},
};

use crate::app::error::Error;

use iced::{container, Column, Container, Length, Row};

pub fn navbar_warning<'a, T: 'a>(warning: Option<&Error>) -> Option<Container<'a, T>> {
    if let Some(e) = warning {
        return Some(card::alert_warning(Container::new(Text::new(&format!(
            "{}",
            e
        )))));
    }
    None
}

pub fn dashboard<'a, T: 'a>(
    header: Container<'a, T>,
    sidebar: Container<'a, T>,
    main: Container<'a, T>,
) -> Container<'a, T> {
    Container::new(
        Column::new()
            .push(header)
            .push(
                Row::new()
                    .push(sidebar.width(Length::Shrink).height(Length::Fill))
                    .push(main.width(Length::Fill).height(Length::Fill)),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill),
    )
}

pub fn sidebar<'a, T: 'a>(menu: Container<'a, T>, footer: Container<'a, T>) -> Container<'a, T> {
    Container::new(
        Column::new()
            .padding(10)
            .push(menu.height(Length::Fill))
            .push(footer.height(Length::Shrink)),
    )
    .style(SidebarStyle)
}

pub struct SidebarStyle;
impl container::StyleSheet for SidebarStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::FOREGROUND.into(),
            border_width: 1.0,
            border_color: color::SECONDARY,
            ..container::Style::default()
        }
    }
}

pub struct SidebarMenuStyle;
impl container::StyleSheet for SidebarMenuStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::FOREGROUND.into(),
            ..container::Style::default()
        }
    }
}

pub fn sidebar_menu<'a, T: 'a>(items: Vec<Container<'a, T>>) -> Container<'a, T> {
    let mut col = Column::new().padding(15).spacing(15);
    for i in items {
        col = col.push(i)
    }
    Container::new(col).style(SidebarMenuStyle)
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
