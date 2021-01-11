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

use crate::revaultd::model::{BroadcastedTransaction, Vault, VaultTransactions};

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
        if let Some((vlt, txs)) = &self.vault {
            let tx = txs.last_broadcasted_tx();
            Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(
                            Row::new().push(Column::new().width(Length::Fill)).push(
                                Container::new(button::cancel(
                                    &mut self.cancel_button,
                                    Container::new(Text::new("X Close")).padding(10),
                                    Message::SelectVault(vlt.outpoint()),
                                ))
                                .width(Length::Shrink),
                            ),
                        )
                        .push(
                            Container::new(Text::new("Transaction Details"))
                                .width(Length::Fill)
                                .align_x(Align::Center),
                        )
                        .push(
                            card::simple(Container::new(
                                Column::new()
                                    .push(
                                        Row::new()
                                            .push(
                                                Container::new(
                                                    Row::new()
                                                        .push(badge::tx_deposit())
                                                        .push(
                                                            Column::new()
                                                                .push(text::small(&vlt.txid))
                                                                .push(text::small("datetime")),
                                                        )
                                                        .spacing(20),
                                                )
                                                .width(Length::Fill),
                                            )
                                            .push(
                                                Container::new(Text::new(format!(
                                                    "{}",
                                                    vlt.amount as f64 / 100000000_f64
                                                )))
                                                .width(Length::Shrink),
                                            )
                                            .spacing(20)
                                            .align_items(Align::Center),
                                    )
                                    .push(separation().width(Length::Fill))
                                    .push(
                                        Row::new()
                                            .push(
                                                Container::new(
                                                    Column::new()
                                                        .push(Text::new("Blockheight"))
                                                        .push(Text::new(
                                                            if let Some(blockheight) =
                                                                &tx.blockheight
                                                            {
                                                                format!("{}", blockheight)
                                                            } else {
                                                                "Not in a block".to_string()
                                                            },
                                                        )),
                                                )
                                                .width(Length::FillPortion(2)),
                                            )
                                            .push(
                                                Container::new(
                                                    Column::new().push(Text::new("Fee")),
                                                )
                                                .width(Length::FillPortion(2)),
                                            ),
                                    )
                                    .spacing(20),
                            ))
                            .width(Length::Fill)
                            .align_x(Align::Center)
                            .padding(20),
                        )
                        .push(
                            input_and_outputs(&tx)
                                .width(Length::Fill)
                                .align_x(Align::Center),
                        )
                        .spacing(20),
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

fn input_and_outputs<'a, T: 'a>(broadcasted: &BroadcastedTransaction) -> Container<'a, T> {
    let mut col_input = Column::new().push(Text::new("Inputs")).spacing(10);
    for input in &broadcasted.tx.input {
        col_input = col_input.push(card::simple(Container::new(text::small(&format!(
            "{}",
            input.previous_output
        )))));
    }
    let mut col_output = Column::new().push(Text::new("Outputs")).spacing(10);
    for output in &broadcasted.tx.output {
        col_output = col_output.push(card::simple(Container::new(text::small(&format!(
            "{} {}",
            output.script_pubkey,
            output.value as f64 / 100000000_f64,
        )))));
    }
    Container::new(Row::new().push(col_input).push(col_output).spacing(20))
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

    pub fn load(&mut self, vaults: Vec<Rc<Vault>>) {
        self.0 = Vec::new();
        for vlt in vaults {
            self.0.push(VaultListItem::new(vlt));
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
            vault,
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
