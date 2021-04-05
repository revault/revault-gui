use std::collections::HashMap;

use iced::{scrollable, Align, Column, Container, Element, Length, Row};

use crate::{
    revault::Role,
    revaultd::model::VaultStatus,
    ui::{
        component::{button, card, navbar, scroll, separation, text},
        error::Error,
        icon::{person_check_icon, shield_check_icon, shield_notif_icon},
        menu::Menu,
        message::Message,
        view::{layout, sidebar::Sidebar, Context},
    },
};

#[derive(Debug)]
pub struct ManagerHomeView {
    sidebar: Sidebar,
    overview: Overview,
    scroll: scrollable::State,
    deposit_button: iced::button::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
            overview: Overview::new(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        spend_txs: Vec<Element<'a, Message>>,
        moving_vaults: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<'a, Message> {
        let mut content = Column::new().push(self.overview.view(ctx, balance));

        if !spend_txs.is_empty() {
            content = content.push(
                Column::new()
                    .push(
                        Column::new()
                            .push(text::bold(text::simple("Pending spend transactions")))
                            .push(text::small(
                                "These transactions are waiting for managers signatures",
                            )),
                    )
                    .push(Column::with_children(spend_txs).spacing(10))
                    .spacing(20),
            )
        }

        if balance.len() == 0 {
            content = content.push(card::simple(Container::new(
                Row::new()
                    .push(
                        Container::new(text::simple(
                            "No vaults yet, start using Revault by making a deposit",
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        button::primary(
                            &mut self.deposit_button,
                            button::button_content(None, "Deposit"),
                        )
                        .on_press(Message::Menu(Menu::Deposit)),
                    )
                    .align_items(iced::Align::Center),
            )))
        }

        if !moving_vaults.is_empty() {
            content = content
                .push(text::bold(text::simple("Funds are moving:")))
                .push(Column::with_children(moving_vaults).spacing(10))
                .spacing(20)
        };

        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(content.spacing(20)),
            ))),
        )
        .into()
    }
}

#[derive(Debug)]
pub struct StakeholderHomeView {
    sidebar: Sidebar,
    overview: Overview,
    scroll: scrollable::State,
    ack_fund_button: iced::button::State,
    deposit_button: iced::button::State,
}

impl StakeholderHomeView {
    pub fn new() -> Self {
        StakeholderHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
            overview: Overview::new(),
            ack_fund_button: iced::button::State::default(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        moving_vaults: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<'a, Message> {
        let mut col_body = Column::new().push(self.overview.view(ctx, balance));
        if balance.len() == 0 {
            col_body = col_body.push(card::simple(Container::new(
                Row::new()
                    .push(
                        Container::new(text::simple(
                            "No vaults yet, start using Revault by making a deposit",
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        button::primary(
                            &mut self.deposit_button,
                            button::button_content(None, "Deposit"),
                        )
                        .on_press(Message::Menu(Menu::Deposit)),
                    )
                    .align_items(iced::Align::Center),
            )))
        }

        if !moving_vaults.is_empty() {
            col_body = col_body
                .push(text::bold(text::simple("Funds are moving:")))
                .push(Column::with_children(moving_vaults).spacing(10))
                .spacing(20)
        };

        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(col_body.spacing(20)),
            ))),
        )
        .into()
    }
}

#[derive(Debug)]
struct Overview {
    ack_fund_button: iced::button::State,
}

impl Overview {
    pub fn new() -> Self {
        Self {
            ack_fund_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        ctx: &Context,
        overview: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<Message> {
        let (nb_total_vaults, total_amount) =
            overview.iter().fold((0, 0), |acc, (_, (nb, amount))| {
                (acc.0 + nb, acc.1 + amount)
            });

        let mut col_body = Column::new()
            .push(text::bold(text::simple("overview:")))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(Column::new().width(Length::Fill))
                            .push(
                                text::bold(text::simple(
                                    &ctx.converter.converts(total_amount).to_string(),
                                ))
                                .size(50),
                            )
                            .push(text::simple(&format!(" {}", ctx.converter.unit)))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Column::new().width(Length::Fill))
                            .push(text::bold(text::simple(&format!("{}", nb_total_vaults))))
                            .push(text::simple(" vaults")),
                    ),
            );

        if let Some((nb_active_vaults, active_amount)) = overview.get(&VaultStatus::Active) {
            col_body = col_body.push(active_funds_overview_card(
                ctx,
                *nb_active_vaults,
                *active_amount,
                overview.get(&VaultStatus::Activating),
            ));
        } else if overview.get(&VaultStatus::Activating).is_some() {
            col_body = col_body.push(active_funds_overview_card(
                ctx,
                0,
                0,
                overview.get(&VaultStatus::Activating),
            ));
        }

        if let Some((nb_secured_vaults, secured_amount)) = overview.get(&VaultStatus::Secured) {
            col_body = col_body.push(acked_funds_overview_card(
                ctx,
                *nb_secured_vaults,
                *secured_amount,
                overview.get(&VaultStatus::Securing),
            ));
        } else if overview.get(&VaultStatus::Securing).is_some() {
            col_body = col_body.push(acked_funds_overview_card(
                ctx,
                0,
                0,
                overview.get(&VaultStatus::Securing),
            ));
        }

        if let Some((nb_funded_vaults, funded_amount)) = overview.get(&VaultStatus::Funded) {
            if ctx.role == Role::Stakeholder {
                col_body = col_body.push(
                    Column::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(text::bold(text::simple(&format!(
                                        "{}",
                                        ctx.converter.converts(*funded_amount),
                                    ))))
                                    .push(text::simple(&format!(
                                        " {} received in ",
                                        ctx.converter.unit
                                    )))
                                    .push(text::bold(text::simple(&nb_funded_vaults.to_string())))
                                    .push(text::simple(" new deposits since last signing")),
                            )
                            .width(Length::Fill)
                            .align_x(iced::Align::End),
                        )
                        .push(
                            Container::new(
                                button::important(
                                    &mut self.ack_fund_button,
                                    button::button_content(
                                        Some(shield_notif_icon()),
                                        "Acknowledge funds",
                                    ),
                                )
                                .on_press(Message::Menu(Menu::ACKFunds)),
                            )
                            .width(Length::Fill)
                            .align_x(iced::Align::End),
                        )
                        .spacing(10)
                        .width(Length::Fill),
                );
            } else {
                col_body = col_body.push(
                    Column::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(text::bold(text::simple(&format!(
                                        "{}",
                                        ctx.converter.converts(*funded_amount),
                                    ))))
                                    .push(text::simple(&format!(
                                        " {} received in ",
                                        ctx.converter.unit
                                    )))
                                    .push(text::bold(text::simple(&nb_funded_vaults.to_string())))
                                    .push(text::simple(
                                        " new deposits, waiting stakeholders acknowledgment",
                                    )),
                            )
                            .width(Length::Fill)
                            .align_x(iced::Align::End),
                        )
                        .spacing(10)
                        .width(Length::Fill),
                );
            }
        }
        card::white(Container::new(col_body.spacing(15))).into()
    }
}

fn active_funds_overview_card<'a, T: 'a>(
    ctx: &Context,
    nb_active_vaults: u64,
    active_amount: u64,
    activating: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let mut col = Column::new().push(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(person_check_icon())
                        .push(text::simple("  Active funds:"))
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                text::bold(text::simple(
                    &ctx.converter.converts(active_amount).to_string(),
                ))
                .size(20),
            )
            .push(text::simple(&format!(" {},   ", ctx.converter.unit)))
            .push(text::bold(text::simple(&nb_active_vaults.to_string())))
            .push(text::simple(" vaults"))
            .align_items(Align::End),
    );

    if let Some((nb_activating_vaults, activating_amount)) = activating {
        col = col.push(separation().width(Length::Fill)).push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(text::small("funds waiting other stakeholder signatures:"))
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill),
                )
                .push(text::bold(text::small("+ ")))
                .push(text::bold(text::small(
                    &ctx.converter.converts(*activating_amount).to_string(),
                )))
                .push(text::small(&format!(" {},   ", ctx.converter.unit)))
                .push(text::bold(text::small(&nb_activating_vaults.to_string())))
                .push(text::small(" vaults"))
                .align_items(Align::End),
        )
    }

    card::border_primary(Container::new(col.spacing(5)))
}

fn acked_funds_overview_card<'a, T: 'a>(
    ctx: &Context,
    nb_secured_vaults: u64,
    secured_amount: u64,
    securing: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let mut col = Column::new().push(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(shield_check_icon())
                        .push(text::simple("  Acknowledged funds:"))
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                text::bold(text::simple(
                    &ctx.converter.converts(secured_amount).to_string(),
                ))
                .size(20),
            )
            .push(text::simple(&format!(" {},   ", ctx.converter.unit)))
            .push(text::bold(text::simple(&nb_secured_vaults.to_string())))
            .push(text::simple(" vaults"))
            .align_items(Align::End),
    );

    if let Some((nb_securing_vaults, securing_amount)) = securing {
        col = col.push(separation().width(Length::Fill)).push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(text::small("funds waiting other stakeholder signatures:"))
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill),
                )
                .push(text::bold(text::small("+ ")))
                .push(text::bold(text::small(
                    &ctx.converter.converts(*securing_amount).to_string(),
                )))
                .push(text::small(&format!(" {},   ", ctx.converter.unit)))
                .push(text::bold(text::small(&nb_securing_vaults.to_string())))
                .push(text::small(" vaults"))
                .align_items(Align::End),
        )
    }
    card::border_secondary(Container::new(col.spacing(5)))
}
