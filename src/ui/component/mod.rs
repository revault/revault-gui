pub mod button;

use super::color;
use iced::{container, scrollable, Column, Container, Length, Row, Scrollable};

use crate::ui::image::revault_colored_logo;

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

pub mod card {
    use crate::ui::color;
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

    pub fn border_secondary<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content)
            .padding(15)
            .style(BorderSecondaryCardStyle)
    }

    pub struct BorderSecondaryCardStyle;
    impl container::StyleSheet for BorderSecondaryCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_color: color::SECONDARY,
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

pub mod text {
    use crate::ui::{color, font};
    use iced::{Container, Text};

    pub fn simple(content: &str) -> Text {
        Text::new(content).font(font::REGULAR).size(20)
    }

    pub fn small(content: &str) -> Text {
        Text::new(content).font(font::REGULAR).size(15)
    }

    pub fn paragraph<'a, T: 'a>(s: &str) -> Container<'a, T> {
        Container::new(Text::new(s).font(font::REGULAR))
    }

    pub fn bold(t: Text) -> Text {
        t.font(font::BOLD)
    }

    pub fn success(t: Text) -> Text {
        t.color(color::SUCCESS)
    }

    pub fn danger(t: Text) -> Text {
        t.color(color::PRIMARY)
    }
}

pub mod badge {
    use crate::ui::{
        color,
        icon::{
            block_icon, deposit_icon, send_icon, shield_check_icon, shield_icon, shield_notif_icon,
            turnback_icon,
        },
    };
    use iced::{container, Container, Length};

    pub fn shield<'a, T: 'a>() -> Container<'a, T> {
        let icon = shield_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(ShieldBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct ShieldBadgeStyle;
    impl container::StyleSheet for ShieldBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::FOREGROUND.into(),
                text_color: color::CANCEL.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn shield_success<'a, T: 'a>() -> Container<'a, T> {
        let icon = shield_check_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(ShieldSuccessBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct ShieldSuccessBadgeStyle;
    impl container::StyleSheet for ShieldSuccessBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::FOREGROUND.into(),
                text_color: color::SUCCESS.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn shield_notif<'a, T: 'a>() -> Container<'a, T> {
        let icon = shield_notif_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(ShieldNotifBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct ShieldNotifBadgeStyle;
    impl container::StyleSheet for ShieldNotifBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::FOREGROUND.into(),
                text_color: color::CANCEL.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn block<'a, T: 'a>() -> Container<'a, T> {
        let icon = block_icon().width(Length::Units(20));
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
                text_color: color::PRIMARY.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn tx_deposit<'a, T: 'a>() -> Container<'a, T> {
        let icon = deposit_icon().width(Length::Units(20));
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
                background: color::INFO_LIGHT.into(),
                text_color: color::INFO.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn vault_unconfirmed<'a, T: 'a>() -> Container<'a, T> {
        let icon = deposit_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(WarningBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    pub fn vault_unvaulting<'a, T: 'a>() -> Container<'a, T> {
        let icon = send_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(WarningBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    pub fn vault_canceling<'a, T: 'a>() -> Container<'a, T> {
        let icon = turnback_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(WarningBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    pub fn vault_spending<'a, T: 'a>() -> Container<'a, T> {
        let icon = send_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(WarningBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct WarningBadgeStyle;
    impl container::StyleSheet for WarningBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::WARNING_LIGHT.into(),
                text_color: color::WARNING.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn vault_canceled<'a, T: 'a>() -> Container<'a, T> {
        let icon = turnback_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(AlertBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct AlertBadgeStyle;
    impl container::StyleSheet for AlertBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::WARNING_LIGHT.into(),
                text_color: color::WARNING.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn vault_spent<'a, T: 'a>() -> Container<'a, T> {
        let icon = send_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(SuccessBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct SuccessBadgeStyle;
    impl container::StyleSheet for SuccessBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::SUCCESS_LIGHT.into(),
                text_color: color::SUCCESS.into(),
                ..container::Style::default()
            }
        }
    }

    pub fn pending_spent_tx<'a, T: 'a>() -> Container<'a, T> {
        let icon = send_icon().width(Length::Units(20));
        Container::new(icon)
            .width(Length::Units(40))
            .height(Length::Units(40))
            .style(InactiveBadgeStyle)
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Center)
    }

    struct InactiveBadgeStyle;
    impl container::StyleSheet for InactiveBadgeStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 40.0,
                background: color::BACKGROUND.into(),
                ..container::Style::default()
            }
        }
    }
}
