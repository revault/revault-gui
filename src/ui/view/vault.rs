use chrono::NaiveDateTime;
use iced::{container, scrollable, Align, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    color,
    component::{badge, button, card, separation, text},
    message::Message,
    view::Context,
};

use crate::revaultd::model::{BroadcastedTransaction, Vault, VaultTransactions};

#[derive(Debug)]
pub enum VaultView {
    Modal(VaultModal),
    ListItem(VaultListItem),
}

impl VaultView {
    pub fn new() -> Self {
        Self::ListItem(VaultListItem::new())
    }
    pub fn new_modal() -> Self {
        Self::Modal(VaultModal::new())
    }

    pub fn view(
        &mut self,
        ctx: &Context,
        vault: &Vault,
        txs: &VaultTransactions,
    ) -> Element<Message> {
        match self {
            Self::ListItem(v) => v.view(vault),
            Self::Modal(v) => v.view(ctx, vault, txs),
        }
    }
}

#[derive(Debug)]
pub struct VaultModal {
    copy_button: iced::button::State,
    cancel_button: iced::button::State,
    scroll: scrollable::State,
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            copy_button: iced::button::State::default(),
            cancel_button: iced::button::State::default(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vlt: &Vault,
        txs: &VaultTransactions,
    ) -> Element<'a, Message> {
        let tx = txs.last_broadcasted_tx();
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(button::cancel(
                                &mut self.cancel_button,
                                Container::new(text::simple("X Close")).padding(10),
                                Message::SelectVault(vlt.outpoint()),
                            ))
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(text::simple("Transaction Details"))
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .push(
                        Container::new(
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
                                                                .push(
                                                                    Row::new()
                                                                        .push(text::small(
                                                                            &vlt.txid,
                                                                        ))
                                                                        .push(button::clipboard(
                                                                            &mut self.copy_button,
                                                                            Message::Clipboard(
                                                                                vlt.txid
                                                                                    .to_string(),
                                                                            ),
                                                                        ))
                                                                        .align_items(Align::Center),
                                                                )
                                                                .push(text::small(&format!(
                                                                    "{}",
                                                                    NaiveDateTime::from_timestamp(
                                                                        tx.received_at,
                                                                        0
                                                                    )
                                                                ))),
                                                        )
                                                        .spacing(20),
                                                )
                                                .width(Length::Fill),
                                            )
                                            .push(
                                                Container::new(
                                                    Row::new()
                                                        .push(text::bold(text::simple(&format!(
                                                            "{}",
                                                            vlt.amount as f64 / 100000000_f64
                                                        ))))
                                                        .push(text::simple(" BTC")),
                                                )
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
                                                        .push(text::bold(text::simple(
                                                            "Blockheight",
                                                        )))
                                                        .push(text::simple(&if let Some(
                                                            blockheight,
                                                        ) = &tx.blockheight
                                                        {
                                                            format!("{}", blockheight)
                                                        } else {
                                                            "Not in a block".to_string()
                                                        })),
                                                )
                                                .width(Length::FillPortion(2)),
                                            )
                                            .push(
                                                Container::new(
                                                    Column::new()
                                                        .push(text::bold(text::simple("Fee"))),
                                                )
                                                .width(Length::FillPortion(2)),
                                            ),
                                    )
                                    .spacing(20),
                            ))
                            .max_width(1000)
                            .padding(20),
                        )
                        .width(Length::Fill)
                        .align_x(Align::Center),
                    )
                    .push(
                        input_and_outputs(ctx, &tx)
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
        .align_x(Align::Center)
        .into()
    }
}

fn input_and_outputs<'a, T: 'a>(
    ctx: &Context,
    broadcasted: &BroadcastedTransaction,
) -> Container<'a, T> {
    let mut col_input = Column::new()
        .push(text::bold(text::simple("Inputs")))
        .spacing(10);
    for input in &broadcasted.tx.input {
        col_input = col_input.push(card::simple(Container::new(text::small(&format!(
            "{}",
            input.previous_output
        )))));
    }
    let mut col_output = Column::new()
        .push(text::bold(text::simple("Outputs")))
        .spacing(10);
    for output in &broadcasted.tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network);
        let mut col = Column::new();
        if let Some(a) = addr {
            col = col.push(text::small(&format!("{}", a,)))
        } else {
            col = col.push(text::small(&format!("{}", &output.script_pubkey)))
        }
        col_output = col_output.push(card::simple(Container::new(col.push(text::bold(
            text::small(&format!("{}", output.value as f64 / 100000000_f64)),
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
pub struct VaultListItem {
    state: iced::button::State,
}

impl VaultListItem {
    pub fn new() -> Self {
        VaultListItem {
            state: iced::button::State::new(),
        }
    }

    pub fn view(&mut self, vault: &Vault) -> iced::Element<Message> {
        card::rounded(Container::new(button::transparent(
            &mut self.state,
            card::white(Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::tx_deposit())
                                .push(text::small(&vault.txid))
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    vault.amount as f64 / 100000000_f64
                                ))))
                                .push(text::small(" BTC"))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            )),
            Message::SelectVault(vault.outpoint()),
        )))
        .into()
    }
}
