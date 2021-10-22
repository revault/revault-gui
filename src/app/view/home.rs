use std::collections::HashMap;

use iced::{
    tooltip::{self, Tooltip},
    Align, Column, Container, Element, HorizontalAlignment, Length, Row,
};

use revault_ui::{
    component::{button, card, text::Text, TooltipStyle},
    icon::{history_icon, person_check_icon, shield_check_icon, tooltip_icon},
};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message, view::layout},
    daemon::{client::Client, model::VaultStatus},
};

#[derive(Debug)]
pub struct ManagerHomeView {
    dashboard: layout::Dashboard,
    deposit_button: iced::button::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            dashboard: layout::Dashboard::new(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        spend_txs: Vec<Element<'a, Message>>,
        moving_vaults: Vec<Element<'a, Message>>,
        active_funds: u64,
        inactive_funds: u64,
    ) -> Element<'a, Message> {
        let mut content = Column::new().push(manager_overview(ctx, active_funds, inactive_funds));

        if !spend_txs.is_empty() {
            content = content.push(
                Column::new()
                    .push(
                        Column::new()
                            .push(Text::new("Pending spend transactions").bold())
                            .push(
                                Text::new("These transactions are waiting for managers signatures")
                                    .small(),
                            ),
                    )
                    .push(Column::with_children(spend_txs).spacing(10))
                    .spacing(20),
            )
        }

        if active_funds == 0 && inactive_funds == 0 {
            content = content.push(card::simple(
                Row::new()
                    .push(
                        Container::new(Text::new(
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
            ))
        }

        if !moving_vaults.is_empty() {
            content = content
                .push(Text::new("Funds are moving:"))
                .push(Column::with_children(moving_vaults).spacing(10))
        };

        self.dashboard.view(ctx, warning, content.spacing(20))
    }
}

fn manager_overview<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    active_funds: u64,
    inactive_funds: u64,
) -> Container<'a, T> {
    Container::new(
        Column::new().push(
            Column::new()
                .push(
                    Row::new()
                        .push(Column::new().width(Length::Fill))
                        .push(
                            Text::new(&ctx.converter.converts(active_funds).to_string())
                                .bold()
                                .size(50),
                        )
                        .push(Text::new(&format!(" {}", ctx.converter.unit)))
                        .align_items(Align::Center),
                )
                .push(Column::new().padding(5))
                .push(
                    Row::new()
                        .push(Column::new().width(Length::Fill))
                        .push(Text::new(&ctx.converter.converts(inactive_funds).to_string()).bold())
                        .push(Text::new(&format!(" {}", ctx.converter.unit)))
                        .align_items(Align::Center),
                )
                .push(
                    Container::new(
                        Text::new("are held by stakeholders")
                            .horizontal_alignment(HorizontalAlignment::Right)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill),
                ),
        ),
    )
}

#[derive(Debug)]
pub struct StakeholderHomeView {
    dashboard: layout::Dashboard,
    overview: StakeholderOverview,
    ack_fund_button: iced::button::State,
    deposit_button: iced::button::State,
}

impl StakeholderHomeView {
    pub fn new() -> Self {
        StakeholderHomeView {
            dashboard: layout::Dashboard::new(),
            overview: StakeholderOverview::new(),
            ack_fund_button: iced::button::State::default(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        moving_vaults: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<'a, Message> {
        let mut col_body = Column::new().push(self.overview.view(ctx, balance));
        if balance.is_empty() {
            col_body = col_body.push(card::simple(Container::new(
                Row::new()
                    .push(
                        Container::new(Text::new(
                            "No vaults yet, start using Revault by making a deposit",
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        button::primary(
                            &mut self.deposit_button,
                            button::button_content(None, "Deposit"),
                        )
                        .on_press(Message::Menu(Menu::CreateVaults)),
                    )
                    .align_items(iced::Align::Center),
            )))
        }

        if !moving_vaults.is_empty() {
            col_body = col_body
                .push(Text::new("Funds are moving:").bold())
                .push(Column::with_children(moving_vaults).spacing(10))
                .spacing(20)
        };

        self.dashboard.view(ctx, warning, col_body.spacing(20))
    }
}

#[derive(Debug)]
struct StakeholderOverview {
    ack_fund_button: iced::button::State,
    delegate_fund_button: iced::button::State,
}

impl StakeholderOverview {
    pub fn new() -> Self {
        Self {
            ack_fund_button: iced::button::State::new(),
            delegate_fund_button: iced::button::State::new(),
        }
    }

    pub fn view<C: Client>(
        &mut self,
        ctx: &Context<C>,
        overview: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<Message> {
        let (nb_total_vaults, total_amount) =
            overview.iter().fold((0, 0), |acc, (status, (nb, amount))| {
                if *status == VaultStatus::Funded || *status == VaultStatus::Unconfirmed {
                    (acc.0, acc.1)
                } else {
                    (acc.0 + nb, acc.1 + amount)
                }
            });

        let mut col = Column::new();

        if let Some((nb_funded_vaults, funded_amount)) = overview.get(&VaultStatus::Funded) {
            col = col.push(
                card::white(Container::new(
                    Row::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(
                                        Text::new(&format!(
                                            "{}",
                                            ctx.converter.converts(*funded_amount),
                                        ))
                                        .bold(),
                                    )
                                    .push(Text::new(&format!(
                                        " {} received in ",
                                        ctx.converter.unit
                                    )))
                                    .push(Text::new(&nb_funded_vaults.to_string()).bold())
                                    .push(Text::new(" new deposits")),
                            )
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                        )
                        .push(
                            Container::new(
                                Row::new()
                                    .push(
                                        button::primary(
                                            &mut self.ack_fund_button,
                                            button::button_content(None, "+ Create vaults")
                                                .padding(3),
                                        )
                                        .on_press(Message::Menu(Menu::CreateVaults)),
                                    )
                                    .align_items(Align::Center)
                                    .spacing(5),
                            )
                            .width(Length::Shrink)
                            .align_x(Align::Center),
                        )
                        .align_items(Align::Center)
                        .spacing(10),
                ))
                .padding(5),
            )
        }

        col = col.push(
            Column::new()
                .push(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Column::new().width(Length::Fill))
                                .push(
                                    Text::new(&ctx.converter.converts(total_amount).to_string())
                                        .bold()
                                        .size(50),
                                )
                                .push(Text::new(&format!(" {}", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                        .push(
                            Row::new()
                                .push(Column::new().width(Length::Fill))
                                .push(Text::new(&format!("{}", nb_total_vaults)).bold())
                                .push(Text::new(" vaults")),
                        ),
                )
                .push(
                    Row::new()
                        .push(
                            secured_funds_overview_card(
                                ctx,
                                overview.get(&VaultStatus::Secured),
                                overview.get(&VaultStatus::Securing),
                                overview.get(&VaultStatus::Activating),
                            )
                            .width(Length::FillPortion(1)),
                        )
                        .push(
                            active_funds_overview_card(ctx, overview.get(&VaultStatus::Active))
                                .width(Length::FillPortion(1)),
                        )
                        .spacing(10),
                )
                .spacing(20),
        );

        Container::new(col.spacing(40)).into()
    }
}

fn active_funds_overview_card<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    active: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let (nb_active_vaults, active_amount) = active.unwrap_or(&(0, 0));
    let col = Column::new()
        .push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(person_check_icon())
                            .push(Text::new("  Delegated funds").bold())
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill),
                )
                .push(
                    Tooltip::new(
                        tooltip_icon().size(20),
                        "Delegated funds can be spent by managers,\n but you can still revert any undesired transaction.",
                        tooltip::Position::Left,
                    )
                    .gap(5)
                    .size(20)
                    .padding(10)
                    .style(TooltipStyle),
                ),
        )
        .push(
            Column::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(
                                Text::new(
                                    &ctx.converter.converts(*active_amount).to_string(),
                                ).bold()
                            )
                            .push(Text::new(&format!(
                                " {:<6}",
                                // to_string is needed to use format alignment feature
                                ctx.converter.unit.to_string()
                            ))),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .push(
                    Container::new(
                        Row::new()
                            .push(Text::new(&nb_active_vaults.to_string()).bold())
                            .push(Text::new(" vaults")),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                ),
        ).push(Container::new(Row::new().push(Text::new(" ").bold())));
    card::white(Container::new(col.spacing(20)))
}

fn secured_funds_overview_card<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    secure: Option<&(u64, u64)>,
    securing: Option<&(u64, u64)>,
    activating: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let (nb_secured_vaults, secured_amount) = secure.unwrap_or(&(0, 0));
    let (nb_activating_vaults, activating_amount) = activating.unwrap_or(&(0, 0));
    let mut col = Column::new()
        .push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(shield_check_icon())
                            .push(Text::new("  Secured funds").bold())
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill),
                )
                .push(
                    Tooltip::new(
                        tooltip_icon().size(20),
                        "Secured funds are controlled by stakeholders only",
                        tooltip::Position::Left,
                    )
                    .gap(5)
                    .size(20)
                    .padding(10)
                    .style(TooltipStyle),
                ),
        )
        .push(
            Column::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(
                                Text::new(
                                    &ctx.converter
                                        .converts(*secured_amount + *activating_amount)
                                        .to_string(),
                                )
                                .bold(),
                            )
                            .push(Text::new(&format!(
                                " {:<6}",
                                // to_string is needed to use format alignment feature
                                ctx.converter.unit.to_string()
                            ))),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .push(
                    Container::new(
                        Row::new()
                            .push(
                                Text::new(&(nb_secured_vaults + nb_activating_vaults).to_string())
                                    .bold(),
                            )
                            .push(Text::new(" vaults")),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                ),
        );

    if let Some((nb_securing_vaults, securing_amount)) = securing {
        col = col.push(
            Tooltip::new(
                Row::new()
                    .push(Column::new().width(Length::Fill))
                    .push(Text::new("+ ").small().bold())
                    .push(
                        Text::new(&ctx.converter.converts(*securing_amount).to_string())
                            .bold()
                            .small(),
                    )
                    .push(Text::new(&format!(" {}, ", ctx.converter.unit)).small())
                    .push(Text::new(&nb_securing_vaults.to_string()).small().bold())
                    .push(Text::new(" vaults ").small())
                    .push(history_icon().size(20))
                    .align_items(Align::End),
                "Waiting for other stakeholders' signatures",
                tooltip::Position::Bottom,
            )
            .gap(5)
            .size(20)
            .padding(10)
            .style(TooltipStyle),
        )
    } else {
        // An empty column is created in order to ensure the same card height.
        col = col.push(Container::new(Row::new().push(Text::new(" ").small())));
    }
    card::white(Container::new(col.spacing(20)))
}
