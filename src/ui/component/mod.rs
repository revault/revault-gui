pub mod badge;
pub mod button;
pub mod form;
pub mod image;
pub mod text;

use super::{color, font, icon};

use iced::{container, scrollable, Column, Container, Length, Row, Scrollable};

use image::revault_colored_logo;

use std::cmp::Ordering;

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
                background: color::FOREGROUND.into(),
                text_color: color::SUCCESS.into(),
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

    pub fn border_success<'a, T: 'a>(content: Container<'a, T>) -> Container<'a, T> {
        Container::new(content)
            .padding(15)
            .style(BorderSuccessCardStyle)
    }

    pub struct BorderSuccessCardStyle;
    impl container::StyleSheet for BorderSuccessCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                border_color: color::SUCCESS,
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
                background: color::FOREGROUND.into(),
                border_color: color::WARNING_LIGHT.into(),
                ..container::Style::default()
            }
        }
    }

    pub struct ProgressBarDoneCardStyle;
    impl container::StyleSheet for ProgressBarDoneCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                background: color::PRIMARY.into(),
                ..container::Style::default()
            }
        }
    }

    // An empty card used as decoration in the progress bar
    pub fn progress_done<'a, T: 'a>() -> Container<'a, T> {
        Container::new(iced::Row::new()).style(ProgressBarDoneCardStyle)
    }

    pub struct ProgressBarDoingCardStyle;
    impl container::StyleSheet for ProgressBarDoingCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                background: color::PRIMARY_LIGHT.into(),
                ..container::Style::default()
            }
        }
    }

    // An empty card used as decoration in the progress bar
    pub fn progress_doing<'a, T: 'a>() -> Container<'a, T> {
        Container::new(iced::Row::new()).style(ProgressBarDoingCardStyle)
    }

    pub struct ProgressBarTodoCardStyle;
    impl container::StyleSheet for ProgressBarTodoCardStyle {
        fn style(&self) -> container::Style {
            container::Style {
                border_radius: 10.0,
                background: color::SECONDARY.into(),
                ..container::Style::default()
            }
        }
    }

    // An empty card used as decoration in the progress bar
    pub fn progress_todo<'a, T: 'a>() -> Container<'a, T> {
        Container::new(iced::Row::new()).style(ProgressBarTodoCardStyle)
    }
}

pub struct ProgressBar {
    steps: Vec<&'static str>,
}

impl ProgressBar {
    pub fn spend_bar() -> Self {
        Self {
            steps: vec!["Add recipients", "Select fee", "Select coins", "Sign"],
        }
    }

    pub fn draw<'a, T: 'a + Clone>(&self, current_step: usize) -> Container<'a, T> {
        let mut row = Row::new().spacing(20);
        for (i, step) in self.steps.iter().enumerate() {
            match i.cmp(&current_step) {
                Ordering::Less => row = row.push(self.progress_circle_done(step)),
                Ordering::Equal => row = row.push(self.progress_circle_doing(step)),
                Ordering::Greater => row = row.push(self.progress_circle_todo(step)),
            }
        }
        Container::new(row)
    }

    fn progress_circle_todo<'a, T: 'a + Clone>(&self, step: &'static str) -> Container<'a, T> {
        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(icon::todo_icon().color(color::DARK_GREY))
                        .push(text::small(step).color(color::DARK_GREY))
                        .spacing(10)
                        .align_items(iced::Align::Center),
                )
                .push(
                    card::progress_todo()
                        .height(Length::Units(5))
                        .width(Length::Fill),
                )
                .width(Length::Units(120))
                .align_items(iced::Align::Center)
                .spacing(10),
        )
    }

    fn progress_circle_doing<'a, T: 'a + Clone>(&self, step: &'static str) -> Container<'a, T> {
        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(icon::todo_icon().color(color::DARK_GREY))
                        .push(text::small(step).color(color::DARK_GREY))
                        .spacing(10)
                        .align_items(iced::Align::Center),
                )
                .push(
                    card::progress_doing()
                        .height(Length::Units(5))
                        .width(Length::Fill),
                )
                .width(Length::Units(120))
                .align_items(iced::Align::Center)
                .spacing(10),
        )
    }

    fn progress_circle_done<'a, T: 'a + Clone>(&self, step: &'static str) -> Container<'a, T> {
        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(icon::done_icon())
                        .push(text::small(step))
                        .spacing(10)
                        .align_items(iced::Align::Center),
                )
                .push(
                    card::progress_done()
                        .height(Length::Units(5))
                        .width(Length::Fill),
                )
                .width(Length::Units(120))
                .align_items(iced::Align::Center)
                .spacing(10),
        )
    }
}
