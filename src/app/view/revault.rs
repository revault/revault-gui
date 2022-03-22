use iced::{Align, Column, Container, Element, Length, Row};

use bitcoin::Amount;
use revault_ui::{
    color,
    component::{badge, button, text::Text, ContainerForegroundStyle},
    util::Collection,
};

use crate::app::{context::Context, error::Error, menu::Menu, message::Message, view::layout};

use crate::daemon::model::{outpoint, Vault};

#[derive(Debug, Default)]
pub struct RevaultSelectVaultsView {
    modal: layout::Modal,
    next_button: iced::button::State,
}

impl RevaultSelectVaultsView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        selected: (usize, u64),
        vaults: Vec<Element<'a, Message>>,
        total: u64,
        warning: Option<&Error>,
        processing: bool,
    ) -> Element<'a, Message> {
        let col = Column::new()
            .push(
                Column::new()
                    .push(Text::new("Cancel the movement of funds").bold().size(50))
                    .spacing(5),
            )
            .push(
                Column::new().push(
                    Row::new()
                        .push(
                            Text::new(&ctx.converter.converts(Amount::from_sat(total)))
                                .bold()
                                .size(30),
                        )
                        .push(Text::new(&format!(" {} are moving", ctx.converter.unit)))
                        .align_items(Align::Center),
                ),
            )
            .push(
                Column::new()
                    .push(Text::new("Select vaults to revault").width(Length::Fill))
                    .push(Column::with_children(vaults).spacing(5))
                    .width(Length::Fill)
                    .spacing(20),
            );

        Column::new().push(self.modal.view(
                ctx,
                warning,
                col.spacing(30).padding(20).max_width(1000),
                Some("By revaulting, you are broadcasting cancel transaction to the network\n, reverting the movement of funds back to the stakeholders wallet."),
                Message::Menu(Menu::Home),
        )).push_maybe(
    if selected.0 > 0 {
        Some(Container::new(
                Row::new()
                .push(
                    Row::new()
                    .push(
                        Text::new(
                            &ctx.converter
                            .converts(Amount::from_sat(selected.1))
                        )
                        .bold(),
                    )
                    .push(Text::new(&format!(" {} (", ctx.converter.unit)))
                    .push(Text::new(&format!("{}", selected.0)).bold())
                    .push(Text::new(" vaults)"))
                    .width(Length::Fill),
                )
                .push(
                        Container::new(
                            if processing {
                                button::primary(
                                    &mut self.next_button,
                                    button::button_content(None, "Revault"),
                                )
                                    .width(Length::Units(200))
                            } else {
                                button::primary(
                                    &mut self.next_button,
                                    button::button_content(None, "Revault"),
                                )
                                    .on_press(Message::Revault)
                                    .width(Length::Units(200))
                            }
                        )
                            .width(Length::Shrink)
        )
            .align_items(Align::Center)
            .max_width(1000),
        )
            .padding(30)
            .width(Length::Fill)
            .align_x(Align::Center)
            .style(ContainerForegroundStyle),
        )
    } else {None}).into()
    }
}

#[derive(Debug, Default)]
pub struct RevaultSuccessView {
    modal: layout::Modal,
}

impl RevaultSuccessView {
    pub fn view<'a>(&'a mut self, ctx: &Context, total: usize) -> Element<'a, Message> {
        let col = Column::new()
            .push(
                Text::new("Canceling the movement of funds")
                    .success()
                    .bold()
                    .size(50),
            )
            .push(
                Text::new(&format!("{} vaults are revaulting", total))
                    .success()
                    .bold(),
            )
            .align_items(Align::Center)
            .spacing(30)
            .padding(20)
            .max_width(1000);

        self.modal.view(
                ctx,
                None,
                col,
                Some("By revaulting, you are broadcasting cancel transaction to the network\n, reverting the movement of funds back to the stakeholders wallet."),
                Message::Menu(Menu::Home),
        ).into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RevaultVaultListItemView {
    select_button: iced::button::State,
}

impl RevaultVaultListItemView {
    pub fn view(&mut self, ctx: &Context, vault: &Vault, selected: bool) -> iced::Element<Message> {
        let content = Container::new(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(if selected {
                                badge::circle_dot()
                            } else {
                                badge::circle()
                            })
                            .spacing(20)
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill),
                )
                .push(
                    Container::new(if selected {
                        Row::new()
                            .push(
                                Text::new(&ctx.converter.converts(vault.amount))
                                    .bold()
                                    .color(color::PRIMARY),
                            )
                            .push(
                                Text::new(&format!(" {}", ctx.converter.unit))
                                    .small()
                                    .color(color::PRIMARY),
                            )
                            .align_items(Align::Center)
                    } else {
                        Row::new()
                            .push(Text::new(&ctx.converter.converts(vault.amount)).bold())
                            .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                            .align_items(Align::Center)
                    })
                    .width(Length::Shrink),
                )
                .spacing(20)
                .align_items(Align::Center),
        );

        button::white_card_button(&mut self.select_button, content)
            .on_press(Message::SelectVault(outpoint(vault)))
            .into()
    }
}
