use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{scrollable, Align, Column, Container, Element, Length, Row, Scrollable};

use crate::{
    revaultd::model,
    ui::{
        component::{badge, button, card, text, ContainerBackgroundStyle},
        error::Error,
        menu::Menu,
        message::{Message, SpendTxMessage},
        view::{manager::spend_tx_with_feerate_view, Context},
    },
};

#[derive(Debug)]
pub struct SpendTransactionView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    psbt_input: iced::text_input::State,
    import_button: iced::button::State,
}

impl SpendTransactionView {
    pub fn new() -> Self {
        SpendTransactionView {
            cancel_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            psbt_input: iced::text_input::State::new(),
            import_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        psbt: &Psbt,
        spent_vaults: &[model::Vault],
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().spacing(20);
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))))
        }
        col = col.push(spend_tx_with_feerate_view(ctx, spent_vaults, psbt, None));
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(
                                button::cancel(
                                    &mut self.cancel_button,
                                    Container::new(text::simple("X Close")).padding(10),
                                )
                                .on_press(Message::Menu(Menu::Home)),
                            )
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(col)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionListItemView {
    select_button: iced::button::State,
}

impl SpendTransactionListItemView {
    pub fn new() -> Self {
        Self {
            select_button: iced::button::State::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context, tx: &model::SpendTx) -> Element<SpendTxMessage> {
        let amount = tx
            .psbt
            .global
            .unsigned_tx
            .output
            .iter()
            .fold(0, |acc, output| acc + output.value);
        button::white_card_button(
            &mut self.select_button,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::pending_spent_tx())
                                .push(Column::new().push(text::bold(text::small(&format!(
                                    "txid: {}",
                                    tx.psbt.global.unsigned_tx.txid().to_string()
                                )))))
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(amount),
                                ))))
                                .push(text::small(&format!(" {}", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(SpendTxMessage::Select(tx.psbt.clone()))
        .width(Length::Fill)
        .into()
    }
}
