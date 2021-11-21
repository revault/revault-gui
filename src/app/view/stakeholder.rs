use iced::{Align, Column, Container, Element, Length, ProgressBar, Row};

use revault_ui::{
    component::{button, card, text::Text},
    icon,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{Message, SignMessage},
        view::layout,
    },
    daemon::{
        client::Client,
        model::{Vault, VaultStatus},
    },
};

#[derive(Debug)]
pub struct StakeholderCreateVaultsView {
    modal: layout::Modal,
    sign_button: iced::button::State,
}

impl StakeholderCreateVaultsView {
    pub fn new() -> Self {
        StakeholderCreateVaultsView {
            modal: layout::Modal::new(),
            sign_button: iced::button::State::default(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        deposits: &Vec<Vault>,
        processing: bool,
        hw_connected: bool,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        if deposits.len() == 0
            || !deposits
                .iter()
                .any(|deposit| deposit.status == VaultStatus::Funded)
        {
            return self.modal.view(ctx, warning, Container::new(card::success(
                        Column::new()
                            .padding(20)
                            .align_items(iced::Align::Center)
                            .spacing(30)
                            .push(Text::from(icon::done_icon().size(80)).width(Length::Fill).success())
                            .push(Column::new()
                                .align_items(Align::Center)
                                .spacing(5)
                                .push(Text::new("You pre-signed the security transactions for all new deposits.").success())
                                .push(Text::new("The deposits will be available as secured Vaults when all the stakeholders have completed this process.").small().success())
                            )
                        )).height(Length::Fill).align_y(Align::Center), None, Message::Menu(Menu::Home));
        }
        let total_amount = deposits.iter().map(|deposit| deposit.amount).sum::<u64>();
        let mut content = Column::new()
            .max_width(1000)
            .padding(20)
            .align_items(Align::Center)
            .push(Text::new("Vault the new deposits").bold().size(50))
            .push(
                Row::new()
                    .spacing(5)
                    .push(Text::new(&format!("{}", deposits.len())).bold())
                    .push(Text::new("new deposits ("))
                    .push(Text::new(&format!("{}", ctx.converter.converts(total_amount))).bold())
                    .push(Text::new("BTC ) will be secured in vaults")),
            )
            .spacing(30);

        if hw_connected {
            if processing {
                let total_secured = deposits
                    .iter()
                    .filter(|deposit| {
                        deposit.status == VaultStatus::Securing
                            || deposit.status == VaultStatus::Secured
                    })
                    .count();
                content = content.push(
                    Column::new()
                        .align_items(Align::Center)
                        .push(ProgressBar::new(
                            0.0..=deposits.len() as f32,
                            total_secured as f32,
                        ))
                        .push(Text::new(&format!(
                            "{}/{} deposits processed",
                            total_secured,
                            deposits.len()
                        ))),
                );
            } else {
                content = content.push(
                    button::primary(
                        &mut self.sign_button,
                        button::button_content(None, " Start signing ").width(Length::Units(200)),
                    )
                    .on_press(Message::Sign(SignMessage::SelectSign)),
                );
            }
        } else {
            content = content.push(
                Row::new()
                    .align_items(iced::Align::Center)
                    .spacing(20)
                    .push(icon::connect_device_icon().size(20))
                    .push(Text::new("Connect hardware wallet")),
            )
        }

        self.modal.view(ctx, warning, Container::new(content).height(Length::Fill).align_y(Align::Center), Some("A vault is a deposit with revocation transactions\nsigned and shared between stakeholders"), Message::Menu(Menu::Home))
    }
}

#[derive(Debug)]
pub struct StakeholderDelegateFundsView {
    modal: layout::Modal,
}

impl StakeholderDelegateFundsView {
    pub fn new() -> Self {
        StakeholderDelegateFundsView {
            modal: layout::Modal::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        active_balance: &u64,
        activating_balance: &u64,
        vaults: Vec<Element<'a, Message>>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .push(
                Column::new()
                    .push(
                        Text::new("Delegate funds to your manager team")
                            .bold()
                            .size(50),
                    )
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Text::new(&ctx.converter.converts(*active_balance).to_string())
                                    .bold()
                                    .size(30),
                            )
                            .push(Text::new(&format!(
                                " {} are allocated to managers",
                                ctx.converter.unit
                            )))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(
                                Text::new(&format!(
                                    "+ {}",
                                    ctx.converter.converts(*activating_balance)
                                ))
                                .bold()
                                .size(20),
                            )
                            .push(Text::new(&format!(
                                " {} are waiting for other stakeholders' approval",
                                ctx.converter.unit
                            )))
                            .align_items(Align::Center),
                    ),
            );

        if !vaults.is_empty() {
            col = col.push(Container::new(
                Column::new()
                    .push(Text::new(" Click on the vaults to delegate:"))
                    .push(Column::with_children(vaults).spacing(5))
                    .spacing(20),
            ))
        } else {
            col = col.push(
                Container::new(Text::new("No more funds can be delegated to managers")).padding(5),
            )
        }

        self.modal.view(
            ctx,
            warning,
            col.spacing(30).padding(20),
            Some("By delegating you allow managers to spend the funds,\n but you can still revert any undesired transaction."),
            Message::Menu(Menu::Home),
        )
    }
}
