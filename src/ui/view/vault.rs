use std::rc::Rc;

use iced::{
    container, scrollable, Align, Column, Container, Element, HorizontalAlignment, Length, Row,
    Scrollable, Text,
};

use crate::ui::{
    color,
    component::{badge, button, card, navbar, separation, text, TransparentPickListStyle},
    error::Error,
    image,
    message::Message,
};

use crate::revaultd::model::{Vault, VaultTransactions};

#[derive(Debug)]
pub struct VaultModal {
    cancel_button: iced::button::State,
    vault: Option<(Rc<Vault>, VaultTransactions)>,
    scroll: scrollable::State,
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            cancel_button: iced::button::State::default(),
            vault: None,
            scroll: scrollable::State::new(),
        }
    }

    pub fn load(&mut self, vault: Option<(Rc<Vault>, VaultTransactions)>) {
        if self.vault.is_none() {
            self.scroll = scrollable::State::new();
        }
        self.vault = vault;
    }

    pub fn view<'a>(&'a mut self, background: Container<'a, Message>) -> Container<'a, Message> {
        if let Some((vlt, _)) = &self.vault {
            Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new().push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(button::cancel(
                                &mut self.cancel_button,
                                Container::new(Text::new("X Close")).padding(10),
                                Message::SelectVault(vlt.outpoint()),
                            ))
                            .width(Length::Shrink),
                        ),
                    ),
                )),
            )
            .style(VaultModalStyle)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
        } else {
            background
        }
    }
}

pub struct VaultModalStyle;
impl container::StyleSheet for VaultModalStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::BACKGROUND.into(),
            ..container::Style::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct VaultList(Vec<VaultListItem>);

impl VaultList {
    pub fn new() -> Self {
        VaultList(Vec::new())
    }

    pub fn load(&mut self, vaults: &Vec<Rc<Vault>>) {
        self.0 = Vec::new();
        for vlt in vaults {
            self.0.push(VaultListItem::new(vlt.clone()));
        }
    }

    pub fn view(&mut self) -> Container<Message> {
        if self.0.len() == 0 {
            return Container::new(Text::new("No vaults yet"));
        }
        let mut col = Column::new();
        for item in self.0.iter_mut() {
            col = col.push(item.view());
        }

        Container::new(col.spacing(10))
    }
}

#[derive(Debug, Clone)]
struct VaultListItem {
    state: iced::button::State,
    vault: Rc<Vault>,
}

impl VaultListItem {
    pub fn new(vault: Rc<Vault>) -> Self {
        VaultListItem {
            state: iced::button::State::new(),
            vault: vault,
        }
    }

    pub fn view<'a>(&'a mut self) -> Container<'a, Message> {
        card::rounded(Container::new(button::transparent(
            &mut self.state,
            card::white(Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::tx_deposit())
                                .push(text::small(&self.vault.txid))
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(Text::new(format!(
                            "{}",
                            self.vault.amount as f64 / 100000000_f64
                        )))
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            )),
            Message::SelectVault(self.vault.outpoint()),
        )))
    }
}
