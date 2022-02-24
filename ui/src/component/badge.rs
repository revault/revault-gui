use crate::{
    color,
    icon::{
        bitcoin_icon, block_icon, circle_check_icon, circle_dot_icon, circle_icon, deposit_icon,
        network_icon, person_check_icon, send_icon, shield_check_icon, shield_icon,
        shield_notif_icon, turnback_icon,
    },
};

use iced::{container, Container, Length};

pub fn badge<'a, T: 'a>(icon: iced::Text) -> Container<'a, T> {
    Container::new(icon.width(Length::Units(20)))
        .width(Length::Units(40))
        .height(Length::Units(40))
        .align_x(iced::Align::Center)
        .align_y(iced::Align::Center)
}

pub fn bitcoin_core<'a, T: 'a>() -> Container<'a, T> {
    badge(bitcoin_icon()).style(BitcoinCoreBadgeStyle)
}

struct BitcoinCoreBadgeStyle;
impl container::StyleSheet for BitcoinCoreBadgeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 30.0,
            background: iced::Color::BLACK.into(),
            text_color: iced::Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}

pub fn network<'a, T: 'a>() -> Container<'a, T> {
    badge(network_icon())
}

pub fn person_check<'a, T: 'a>() -> Container<'a, T> {
    badge(person_check_icon()).style(WhiteBadgeStyle)
}

pub fn circle<'a, T: 'a>() -> Container<'a, T> {
    badge(circle_icon()).style(WhiteBadgeStyle)
}

pub fn circle_dot<'a, T: 'a>() -> Container<'a, T> {
    badge(circle_dot_icon()).style(WhiteBadgeStyle)
}

struct WhiteBadgeStyle;
impl container::StyleSheet for WhiteBadgeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 40.0,
            background: color::FOREGROUND.into(),
            text_color: color::CANCEL.into(),
            ..container::Style::default()
        }
    }
}

pub fn person_check_success<'a, T: 'a>() -> Container<'a, T> {
    badge(person_check_icon()).style(CheckSuccessBadgeStyle)
}

pub fn circle_check_success<'a, T: 'a>() -> Container<'a, T> {
    badge(circle_check_icon()).style(CheckSuccessBadgeStyle)
}

struct CheckSuccessBadgeStyle;
impl container::StyleSheet for CheckSuccessBadgeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 40.0,
            background: color::FOREGROUND.into(),
            text_color: color::SUCCESS.into(),
            ..container::Style::default()
        }
    }
}

pub fn shield<'a, T: 'a>() -> Container<'a, T> {
    badge(shield_icon()).style(ShieldBadgeStyle)
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
    badge(shield_check_icon()).style(ShieldSuccessBadgeStyle)
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
    badge(shield_notif_icon()).style(ShieldNotifBadgeStyle)
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
    badge(block_icon()).style(BlockBadgeStyle)
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
    badge(deposit_icon()).style(TxDepositBadgeStyle)
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
    badge(deposit_icon()).style(WarningBadgeStyle)
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
    badge(turnback_icon()).style(WarningBadgeStyle)
}

pub fn vault_spending<'a, T: 'a>() -> Container<'a, T> {
    badge(send_icon()).style(WarningBadgeStyle)
}

struct WarningBadgeStyle;
impl container::StyleSheet for WarningBadgeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 40.0,
            background: color::ALERT_LIGHT.into(),
            text_color: color::ALERT.into(),
            ..container::Style::default()
        }
    }
}

pub fn vault_canceled<'a, T: 'a>() -> Container<'a, T> {
    badge(turnback_icon()).style(AlertBadgeStyle)
}

struct AlertBadgeStyle;
impl container::StyleSheet for AlertBadgeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 40.0,
            background: color::ALERT_LIGHT.into(),
            text_color: color::ALERT.into(),
            ..container::Style::default()
        }
    }
}

pub fn vault_spent<'a, T: 'a>() -> Container<'a, T> {
    badge(send_icon()).style(SuccessBadgeStyle)
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
    badge(send_icon()).style(InactiveBadgeStyle)
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
