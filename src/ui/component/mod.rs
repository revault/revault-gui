pub mod button;

use super::color;
use iced::{container, Column, Container, Length, Row};

use crate::ui::image::revault_colored_logo;

pub fn navbar<'a, T: 'a>(notification: Option<Container<'a, T>>) -> Container<'a, T> {
    let svg = revault_colored_logo()
        .width(Length::Units(150))
        .height(Length::Fill);
    let mut content = Row::new().push(Container::new(svg).center_x().width(Length::Units(200)));

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
            border_color: color::SECONDARY.into(),
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

pub mod card {
    use crate::ui::color;
    use iced::{container, Container};

    pub fn rounded<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content).style(RoundedCardStyle)
    }

    pub struct RoundedCardStyle;
    impl container::StyleSheet for RoundedCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_width: 1.0,
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
                background: color::BACKGROUND_LIGHT.into(),
                ..container::Style::default()
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

pub mod text {
    use iced::{Container, Text};

    pub fn large_title(content: &str) -> Text {
        Text::new(content).size(50)
    }

    pub fn small(content: &str) -> Text {
        Text::new(content).size(15)
    }

    pub fn paragraph<'a, T: 'a>(s: &str) -> Container<'a, T> {
        Container::new(Text::new(s))
    }
}

pub mod badge {
    use crate::ui::{color, image};
    use iced::{container, Container, Length};

    pub fn block<'a, T: 'a>() -> Container<'a, T> {
        let icon = image::block_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(BlockBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct BlockBadgeStyle;
    impl container::StyleSheet for BlockBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::PRIMARY_LIGHT.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn tx_deposit<'a, T: 'a>() -> Container<'a, T> {
        let icon = image::send_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(TxDepositBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct TxDepositBadgeStyle;
    impl container::StyleSheet for TxDepositBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::SUCCESS_LIGHT.into(),
                ..container::Style::default()
            }
        }
    }
}
