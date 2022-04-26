use bitcoin::Amount;
use std::collections::HashMap;

use iced::{
    alignment,
    tooltip::{self, Tooltip},
    Alignment, Column, Container, Element, Length, Row,
};

use revault_ui::{
    color,
    component::{badge, button, card, text::Text, TooltipStyle},
    icon::{
        history_icon, person_check_icon, shield_check_icon, tooltip_icon, warning_octagon_icon,
    },
    util::Collection,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::{Menu, VaultsMenu},
        message::Message,
        view::layout,
    },
    daemon::model::VaultStatus,
};

#[derive(Debug, Default)]
pub struct ManagerHomeView {
    dashboard: layout::Dashboard,
    history_button: iced::button::State,
    moving_vaults_section: MovingVaultsSection,
}

impl ManagerHomeView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        moving_vaults: Vec<Element<'a, Message>>,
        latest_events: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<'a, Message> {
        let content = Column::new()
            .push(manager_overview(ctx, balance))
            .push_maybe(self.moving_vaults_section.view(ctx, moving_vaults, balance))
            .push_maybe(if !latest_events.is_empty() {
                let length = latest_events.len();
                Some(
                    Column::new()
                        .spacing(10)
                        .push(Text::new("Latest events:").small().bold())
                        .push(Column::with_children(latest_events).spacing(5))
                        .push_maybe(if length >= 5 {
                            Some(
                                Container::new(
                                    button::transparent(
                                        &mut self.history_button,
                                        Container::new(Text::new("See more").small())
                                            .width(iced::Length::Fill)
                                            .center_x(),
                                    )
                                    .on_press(Message::Menu(Menu::History)),
                                )
                                .center_x()
                                .width(Length::Fill),
                            )
                        } else {
                            None
                        }),
                )
            } else {
                None
            });

        self.dashboard.view(ctx, warning, content.spacing(20))
    }
}

fn manager_overview<'a, T: 'a>(
    ctx: &Context,
    balance: &HashMap<VaultStatus, (u64, u64)>,
) -> Container<'a, T> {
    let active_funds = balance.get(&VaultStatus::Active).unwrap_or(&(0, 0)).1;
    let inactive_funds = balance.get(&VaultStatus::Funded).unwrap_or(&(0, 0)).1
        + balance.get(&VaultStatus::Secured).unwrap_or(&(0, 0)).1
        + balance.get(&VaultStatus::Securing).unwrap_or(&(0, 0)).1;
    Container::new(
        Column::new().push(
            Column::new()
                .push(
                    Row::new()
                        .push(Column::new().width(Length::Fill))
                        .push(
                            Text::new(&ctx.converter.converts(Amount::from_sat(active_funds)))
                                .bold()
                                .size(50),
                        )
                        .push(Text::new(&format!(" {}", ctx.converter.unit)))
                        .align_items(Alignment::Center),
                )
                .push(Column::new().padding(5))
                .push(
                    Row::new()
                        .push(Column::new().width(Length::Fill))
                        .push(
                            Text::new(&ctx.converter.converts(Amount::from_sat(inactive_funds)))
                                .bold(),
                        )
                        .push(Text::new(&format!(" {}", ctx.converter.unit)))
                        .align_items(Alignment::Center),
                )
                .push(
                    Container::new(
                        Text::new("are held by stakeholders")
                            .horizontal_alignment(alignment::Horizontal::Right)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill),
                ),
        ),
    )
}

#[derive(Debug, Default)]
pub struct StakeholderHomeView {
    dashboard: layout::Dashboard,
    overview: StakeholderOverview,
    moving_vaults_section: MovingVaultsSection,
    history_button: iced::button::State,
    deposit_button: iced::button::State,
}

impl StakeholderHomeView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        moving_vaults: Vec<Element<'a, Message>>,
        latest_events: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Element<'a, Message> {
        let col_body = Column::new()
            .spacing(20)
            .push(self.overview.view(ctx, balance))
            .push_maybe(if balance.is_empty() && latest_events.is_empty() {
                Some(card::simple(Container::new(
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
                        .align_items(Alignment::Center),
                )))
            } else {
                None
            })
            .push_maybe(self.moving_vaults_section.view(ctx, moving_vaults, balance))
            .push_maybe(if !latest_events.is_empty() {
                let length = latest_events.len();
                Some(
                    Column::new()
                        .spacing(10)
                        .push(Text::new("Latest events:").small().bold())
                        .push(Column::with_children(latest_events).spacing(5))
                        .push_maybe(if length >= 5 {
                            Some(
                                Container::new(
                                    button::transparent(
                                        &mut self.history_button,
                                        Container::new(Text::new("See more").small())
                                            .width(iced::Length::Fill)
                                            .center_x(),
                                    )
                                    .on_press(Message::Menu(Menu::History)),
                                )
                                .center_x()
                                .width(Length::Fill),
                            )
                        } else {
                            None
                        }),
                )
            } else {
                None
            });

        self.dashboard.view(ctx, warning, col_body.spacing(20))
    }
}

#[derive(Debug, Default)]
struct MovingVaultsSection {
    revault_button: iced::button::State,
    canceling_vaults_button: iced::button::State,
}

impl MovingVaultsSection {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        spending_vaults: Vec<Element<'a, Message>>,
        balance: &HashMap<VaultStatus, (u64, u64)>,
    ) -> Option<Element<'a, Message>> {
        if !spending_vaults.is_empty()
            || balance.get(&VaultStatus::Unvaulting).is_some()
            || balance.get(&VaultStatus::Unvaulted).is_some()
            || balance.get(&VaultStatus::Canceling).is_some()
        {
            let mut col_body = Column::new()
                .spacing(10)
                .push(Text::new("Funds are moving:").small().bold());

            let (nb, amount) = match (
                balance.get(&VaultStatus::Unvaulting),
                balance.get(&VaultStatus::Unvaulted),
            ) {
                (None, None) => (0_u64, 0_u64),
                (Some((n, a)), None) => (*n, *a),
                (None, Some((n, a))) => (*n, *a),
                (Some((n1, a1)), Some((n2, a2))) => (*n1 + *n2, *a1 + *a2),
            };
            if nb != 0 {
                col_body = col_body.push(
                    button::white_card_button(
                        &mut self.revault_button,
                        Container::new(
                            Column::new().push(
                                Row::new()
                                    .spacing(20)
                                    .align_items(Alignment::Center)
                                    .push(badge::unlock())
                                    .push(
                                        Row::new()
                                            .align_items(Alignment::Center)
                                            .push(Text::new(&format!("{}", nb)).bold())
                                            .push(if nb != 1 {
                                                Text::new(" vaults ( ")
                                            } else {
                                                Text::new(" vault ( ")
                                            })
                                            .push(
                                                Text::new(&format!(
                                                    "{} ",
                                                    Amount::from_sat(amount).as_btc()
                                                ))
                                                .bold(),
                                            )
                                            .push(
                                                Text::new(&ctx.converter.unit.to_string()).small(),
                                            )
                                            .push(if nb != 1 {
                                                Text::new(" ) are unvaulting")
                                            } else {
                                                Text::new(" ) is unvaulting")
                                            }),
                                    )
                                    .push(
                                        Container::new(
                                            Tooltip::new(
                                                warning_octagon_icon().color(color::ALERT).size(20),
                                                "Something is wrong ? Click to intervene",
                                                tooltip::Position::Left,
                                            )
                                            .gap(5)
                                            .size(20)
                                            .padding(10)
                                            .style(TooltipStyle),
                                        )
                                        .align_x(alignment::Horizontal::Right)
                                        .width(Length::Fill),
                                    ),
                            ),
                        ),
                    )
                    .on_press(Message::Menu(Menu::RevaultVaults))
                    .width(Length::Fill),
                );
            }

            if let Some((nb, amount)) = balance.get(&VaultStatus::Canceling) {
                col_body = col_body.push(
                    button::white_card_button(
                        &mut self.canceling_vaults_button,
                        Container::new(
                            Column::new().push(
                                Row::new()
                                    .spacing(20)
                                    .align_items(Alignment::Center)
                                    .push(badge::vault_canceling())
                                    .push(
                                        Row::new()
                                            .align_items(Alignment::Center)
                                            .push(Text::new(&format!("{}", nb)).bold())
                                            .push(if *nb != 1 {
                                                Text::new(" vaults ( ")
                                            } else {
                                                Text::new(" vault ( ")
                                            })
                                            .push(
                                                Text::new(&format!(
                                                    "{} ",
                                                    Amount::from_sat(*amount).as_btc()
                                                ))
                                                .bold(),
                                            )
                                            .push(
                                                Text::new(&ctx.converter.unit.to_string()).small(),
                                            )
                                            .push(if *nb != 1 {
                                                Text::new(" ) are revaulting")
                                            } else {
                                                Text::new(" ) is revaulting")
                                            }),
                                    ),
                            ),
                        ),
                    )
                    .on_press(Message::Menu(Menu::Vaults(VaultsMenu::Moving)))
                    .width(Length::Fill),
                );
            }

            if !spending_vaults.is_empty() {
                col_body = col_body.push(Column::with_children(spending_vaults).spacing(10));
            }

            Some(col_body.into())
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct StakeholderOverview {
    ack_fund_button: iced::button::State,
}

impl StakeholderOverview {
    pub fn view(
        &mut self,
        ctx: &Context,
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
                                            ctx.converter
                                                .converts(Amount::from_sat(*funded_amount)),
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
                            .center_x(),
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
                                    .align_items(Alignment::Center)
                                    .spacing(5),
                            )
                            .width(Length::Shrink)
                            .center_x(),
                        )
                        .align_items(Alignment::Center)
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
                                    Text::new(
                                        &ctx.converter.converts(Amount::from_sat(total_amount)),
                                    )
                                    .bold()
                                    .size(50),
                                )
                                .push(Text::new(&format!(" {}", ctx.converter.unit)))
                                .align_items(Alignment::Center),
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
                            )
                            .width(Length::FillPortion(1)),
                        )
                        .push(
                            active_funds_overview_card(
                                ctx,
                                overview.get(&VaultStatus::Active),
                                overview.get(&VaultStatus::Activating),
                            )
                            .width(Length::FillPortion(1)),
                        )
                        .spacing(10),
                )
                .spacing(20),
        );

        Container::new(col.spacing(40)).into()
    }
}

fn active_funds_overview_card<'a, T: 'a>(
    ctx: &Context,
    active: Option<&(u64, u64)>,
    activating: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let (nb_active_vaults, active_amount) = active.unwrap_or(&(0, 0));
    let mut col = Column::new()
        .push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(person_check_icon())
                            .push(Text::new("  Delegated funds").bold())
                            .align_items(Alignment::Center),
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
                                    &ctx.converter.converts(Amount::from_sat(*active_amount))
                                ).bold()
                            )
                            .push(Text::new(&format!(
                                " {:<6}",
                                // to_string is needed to use format alignment feature
                                ctx.converter.unit.to_string()
                            ))),
                    )
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Right),
                )
                .push(
                    Container::new(
                        Row::new()
                            .push(Text::new(&nb_active_vaults.to_string()).bold())
                            .push(Text::new(" vaults")),
                    )
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Right),
                ),
        );

    if let Some((nb_activating_vaults, activating_amount)) = activating {
        col = col.push(
            Tooltip::new(
                Row::new()
                    .push(Column::new().width(Length::Fill))
                    .push(Text::new("+ ").small().bold())
                    .push(
                        Text::new(&ctx.converter.converts(Amount::from_sat(*activating_amount)))
                            .bold()
                            .small(),
                    )
                    .push(Text::new(&format!(" {}, ", ctx.converter.unit)).small())
                    .push(Text::new(&nb_activating_vaults.to_string()).small().bold())
                    .push(Text::new(" vaults ").small())
                    .push(history_icon().size(20))
                    .align_items(Alignment::End),
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

fn secured_funds_overview_card<'a, T: 'a>(
    ctx: &Context,
    secure: Option<&(u64, u64)>,
    securing: Option<&(u64, u64)>,
) -> Container<'a, T> {
    let (nb_secured_vaults, secured_amount) = secure.unwrap_or(&(0, 0));
    let mut col = Column::new()
        .push(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(shield_check_icon())
                            .push(Text::new("  Secured funds").bold())
                            .align_items(Alignment::Center),
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
                                    &ctx.converter.converts(Amount::from_sat(*secured_amount)),
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
                    .align_x(alignment::Horizontal::Right),
                )
                .push(
                    Container::new(
                        Row::new()
                            .push(Text::new(&(nb_secured_vaults).to_string()).bold())
                            .push(Text::new(" vaults")),
                    )
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Right),
                ),
        );

    if let Some((nb_securing_vaults, securing_amount)) = securing {
        col = col.push(
            Tooltip::new(
                Row::new()
                    .push(Column::new().width(Length::Fill))
                    .push(Text::new("+ ").small().bold())
                    .push(
                        Text::new(&ctx.converter.converts(Amount::from_sat(*securing_amount)))
                            .bold()
                            .small(),
                    )
                    .push(Text::new(&format!(" {}, ", ctx.converter.unit)).small())
                    .push(Text::new(&nb_securing_vaults.to_string()).small().bold())
                    .push(Text::new(" vaults ").small())
                    .push(history_icon().size(20))
                    .align_items(Alignment::End),
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
