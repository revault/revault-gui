pub mod badge;
pub mod button;
pub mod image;
pub mod text;

use super::{color, font};

use iced::{container, scrollable, Column, Container, Length, Row, Scrollable};

use image::revault_colored_logo;

/// scroll is a wrapper for Scrollable in order to fix a bug from iced 0.3.0
/// scroll add padding to the content in order to give space to the scroll bar.
/// TODO: remove it once https://github.com/hecrj/iced/issues/793 is fixed
pub fn scroll<'a, T: 'a>(
    state: &'a mut scrollable::State,
    content: Container<'a, T>,
) -> Scrollable<'a, T> {
    Scrollable::new(state).push(Container::new(content).padding(10))
}

pub fn navbar<'a, T: 'a>(notification: Option<Container<'a, T>>) -> Container<'a, T> {
    let svg = revault_colored_logo()
        .width(Length::Units(100))
        .height(Length::Fill);
    let mut content = Row::new()
        .push(Column::new().width(Length::Units(10)))
        .push(
            Container::new(svg)
                .padding(5)
                .center_x()
                .width(Length::Shrink),
        );

    if let Some(n) = notification {
        content = content.push(Container::new(n).width(Length::Fill));
    }
    Container::new(content)
        .width(Length::Fill)
        .padding(10)
        .style(NavbarStyle)
        .center_y()
}

pub struct NavbarStyle;
impl container::StyleSheet for NavbarStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_width: 1.0,
            border_color: color::SECONDARY,
            background: color::FOREGROUND.into(),
            ..container::Style::default()
        }
    }
}

pub fn separation<'a, T: 'a>() -> Container<'a, T> {
    Container::new(Column::new().push(iced::Text::new(" ")))
        .style(SepStyle)
        .height(Length::Units(1))
}

pub struct SepStyle;
impl container::StyleSheet for SepStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::SECONDARY.into(),
            ..container::Style::default()
        }
    }
}

pub struct ContainerBackgroundStyle;
impl container::StyleSheet for ContainerBackgroundStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::BACKGROUND.into(),
            ..container::Style::default()
        }
    }
}

pub struct TransparentPickListStyle;
impl iced::pick_list::StyleSheet for TransparentPickListStyle {
    fn active(&self) -> iced::pick_list::Style {
        iced::pick_list::Style {
            background: color::FOREGROUND.into(),
            border_width: 1.0,
            border_radius: 10.0,
            ..iced::pick_list::Style::default()
        }
    }
    fn hovered(&self) -> iced::pick_list::Style {
        iced::pick_list::Style {
            background: color::FOREGROUND.into(),
            border_radius: 10.0,
            ..iced::pick_list::Style::default()
        }
    }
    fn menu(&self) -> iced::pick_list::Menu {
        iced::pick_list::Menu {
            background: color::FOREGROUND.into(),
            ..iced::pick_list::Menu::default()
        }
    }
}

pub struct TooltipStyle;
impl container::StyleSheet for TooltipStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: iced::Color::BLACK.into(),
            background: color::FOREGROUND.into(),
            border_radius: 10.0,
            border_width: 1.0,
            border_color: color::SECONDARY,
            ..container::Style::default()
        }
    }
}

pub mod card {
    use super::color;
    use iced::{container, Container};

    pub fn success<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).padding(15).style(SuccessCardStyle)
    }

    pub struct SuccessCardStyle;
    impl container::StyleSheet for SuccessCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_color: color::SUCCESS,
                background: color::SUCCESS_LIGHT.into(),
                text_color: color::FOREGROUND.into(),
                border_radius: 10.0,
                border_width: 1.0,
            }
        }
    }

    pub fn border_black<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content)
            .padding(15)
            .style(BorderBlackCardStyle)
    }

    pub struct BorderBlackCardStyle;
    impl container::StyleSheet for BorderBlackCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_color: iced::Color::BLACK,
                border_width: 2.0,
                background: color::FOREGROUND.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn border_primary<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content)
            .padding(15)
            .style(BorderPrimaryCardStyle)
    }

    pub struct BorderPrimaryCardStyle;
    impl container::StyleSheet for BorderPrimaryCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_color: color::PRIMARY,
                border_width: 2.0,
                background: color::FOREGROUND.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn grey<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).padding(15).style(GreyCardStyle)
    }

    pub struct GreyCardStyle;
    impl container::StyleSheet for GreyCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_color: color::INFO_LIGHT,
                border_width: 2.0,
                background: color::FOREGROUND.into(),
                text_color: color::INFO_LIGHT.into(),
            }
        }
    }

    pub fn white<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).padding(15).style(WhiteCardStyle)
    }

    pub struct WhiteCardStyle;
    impl container::StyleSheet for WhiteCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                background: color::FOREGROUND.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn simple<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).padding(15).style(SimpleCardStyle)
    }

    pub struct SimpleCardStyle;
    impl container::StyleSheet for SimpleCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                background: color::FOREGROUND.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn alert_warning<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).padding(15).style(WarningCardStyle)
    }

    pub struct WarningCardStyle;
    impl container::StyleSheet for WarningCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                text_color: color::WARNING.into(),
                background: color::WARNING_LIGHT.into(),
                ..container::Style::default()
            }
        }
    }
}
