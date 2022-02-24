use bitcoin::Amount;
use iced::{pick_list, Align, Column, Container, Element, Length, ProgressBar, Row};

use revault_ui::{
    component::{button, card, text::Text, ContainerForegroundStyle, TransparentPickListStyle},
    icon,
    util::Collection,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{Message, SignMessage, VaultFilterMessage},
        view::layout,
    },
    daemon::model::{Vault, VaultStatus},
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

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
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
        let total_amount = deposits
            .iter()
            .map(|deposit| deposit.amount.as_sat())
            .sum::<u64>();
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
                    .push(
                        Text::new(&format!(
                            "{}",
                            ctx.converter.converts(Amount::from_sat(total_amount))
                        ))
                        .bold(),
                    )
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
pub struct StakeholderDelegateVaultsView {
    modal: layout::Modal,
    sign_button: iced::button::State,
}

impl StakeholderDelegateVaultsView {
    pub fn new() -> Self {
        StakeholderDelegateVaultsView {
            modal: layout::Modal::new(),
            sign_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        deposits: &Vec<Vault>,
        processing: bool,
        hw_connected: bool,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        if !deposits
            .iter()
            .any(|deposit| deposit.status == VaultStatus::Secured)
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
                                .push(Text::new("You pre-signed the unvault transactions for all the selected vaults.").success())
                                .push(Text::new("The vaults will be available for the managers when all the stakeholders have completed this process.").small().success())
                            )
                        )).height(Length::Fill).align_y(Align::Center), None, Message::Menu(Menu::Home));
        }
        let total_amount = deposits
            .iter()
            .map(|deposit| deposit.amount.as_sat())
            .sum::<u64>();
        let mut content = Column::new()
            .max_width(1000)
            .padding(20)
            .align_items(Align::Center)
            .push(
                Text::new("Delegate vaults to your manager team")
                    .bold()
                    .size(50),
            )
            .push(
                Row::new()
                    .spacing(5)
                    .push(Text::new(&format!("{}", deposits.len())).bold())
                    .push(Text::new("vaults ("))
                    .push(
                        Text::new(&format!(
                            "{}",
                            ctx.converter.converts(Amount::from_sat(total_amount))
                        ))
                        .bold(),
                    )
                    .push(Text::new("BTC ) will be delegated to managers")),
            )
            .spacing(30);

        if hw_connected {
            if processing {
                let total_active = deposits
                    .iter()
                    .filter(|deposit| {
                        deposit.status == VaultStatus::Activating
                            || deposit.status == VaultStatus::Active
                    })
                    .count();
                content = content.push(
                    Column::new()
                        .align_items(Align::Center)
                        .push(ProgressBar::new(
                            0.0..=deposits.len() as f32,
                            total_active as f32,
                        ))
                        .push(Text::new(&format!(
                            "{}/{} vaults processed",
                            total_active,
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

        self.modal.view(ctx, warning, Container::new(content).height(Length::Fill).align_y(Align::Center), Some("By delegating you allow managers to spend the funds,\n but you can still revert any undesired transaction."), Message::Menu(Menu::Home))
    }
}

#[derive(Debug)]
pub struct StakeholderSelecteVaultsToDelegateView {
    modal: layout::Modal,
    next_button: iced::button::State,
    pick_filter: pick_list::State<DelegateVaultsFilter>,
}

impl StakeholderSelecteVaultsToDelegateView {
    pub fn new() -> Self {
        StakeholderSelecteVaultsToDelegateView {
            modal: layout::Modal::new(),
            next_button: iced::button::State::new(),
            pick_filter: pick_list::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        active_balance: &u64,
        activating_balance: &u64,
        vault_status_filter: &'static [VaultStatus],
        selected: (usize, u64),
        vaults: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let col = Column::new()
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
                                Text::new(
                                    &ctx.converter
                                        .converts(Amount::from_sat(*active_balance))
                                        .to_string(),
                                )
                                .bold()
                                .size(30),
                            )
                            .push(Text::new(&format!(
                                " {} are allocated to managers",
                                ctx.converter.unit
                            )))
                            .align_items(Align::Center),
                    )
                    .push_maybe(if *activating_balance != 0 {
                        Some(
                            Row::new()
                                .push(Text::new("You signed to allocate "))
                                .push(
                                    Text::new(&format!(
                                        "+ {}",
                                        ctx.converter
                                            .converts(Amount::from_sat(*activating_balance))
                                    ))
                                    .bold()
                                    .size(20),
                                )
                                .push(Text::new(&format!(" {} more.", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                    } else {
                        None
                    }),
            )
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .push(if vaults.is_empty() {
                                Text::new("No vaults are available").width(Length::Fill)
                            } else {
                                Text::new("Select vaults to delegate:").width(Length::Fill)
                            })
                            .push(
                                pick_list::PickList::new(
                                    &mut self.pick_filter,
                                    &DelegateVaultsFilter::ALL_FILTERS[..],
                                    Some(DelegateVaultsFilter::new(vault_status_filter)),
                                    |filter| {
                                        Message::FilterVaults(VaultFilterMessage::Status(
                                            filter.statuses(),
                                        ))
                                    },
                                )
                                .text_size(20)
                                .padding(10)
                                .width(Length::Units(200))
                                .style(TransparentPickListStyle),
                            ),
                    )
                    .push(Column::with_children(vaults).spacing(5))
                    .width(Length::Fill)
                    .spacing(20),
            );

        let mut col = Column::new().push(self.modal.view(
            ctx,
            None,
            col.spacing(30).padding(20).max_width(1000),
            Some("By delegating you allow managers to spend the funds,\n but you can still revert any undesired transaction."),
            Message::Menu(Menu::Home),
        ));

        if selected.0 > 0 {
            col = col.push(
                Container::new(
                    Row::new()
                        .push(
                            Row::new()
                                .push(
                                    Text::new(
                                        &ctx.converter
                                            .converts(Amount::from_sat(selected.1))
                                            .to_string(),
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
                                button::primary(
                                    &mut self.next_button,
                                    button::button_content(None, "Next"),
                                )
                                .on_press(Message::Next)
                                .width(Length::Units(200)),
                            )
                            .width(Length::Shrink),
                        )
                        .align_items(Align::Center)
                        .max_width(1000),
                )
                .padding(30)
                .width(Length::Fill)
                .align_x(Align::Center)
                .style(ContainerForegroundStyle),
            )
        }

        col.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegateVaultsFilter {
    All,
    Approved,
    Unapproved,
}

impl DelegateVaultsFilter {
    pub const ALL: [VaultStatus; 2] = [VaultStatus::Secured, VaultStatus::Activating];
    pub const APPROVED: [VaultStatus; 1] = [VaultStatus::Activating];
    pub const UNAPPROVED: [VaultStatus; 1] = [VaultStatus::Secured];

    pub const ALL_FILTERS: [DelegateVaultsFilter; 3] = [
        DelegateVaultsFilter::All,
        DelegateVaultsFilter::Approved,
        DelegateVaultsFilter::Unapproved,
    ];

    pub fn new(statuses: &[VaultStatus]) -> DelegateVaultsFilter {
        if statuses == Self::ALL {
            DelegateVaultsFilter::All
        } else if statuses == Self::APPROVED {
            DelegateVaultsFilter::Approved
        } else {
            DelegateVaultsFilter::Unapproved
        }
    }

    pub fn statuses(&self) -> &'static [VaultStatus] {
        match self {
            Self::All => &Self::ALL,
            Self::Approved => &Self::APPROVED,
            Self::Unapproved => &Self::UNAPPROVED,
        }
    }
}

impl std::fmt::Display for DelegateVaultsFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Approved => write!(f, "Approved"),
            Self::Unapproved => write!(f, "Unapproved"),
        }
    }
}
