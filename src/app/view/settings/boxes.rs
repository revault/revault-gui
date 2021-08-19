use iced::{Align, Column, Container, Length, Row};

use crate::{
    app::message::Message,
    ui::{
        color,
        component::{badge, card, separation, text},
        icon::dot_icon,
    },
};

use crate::revaultd::config::Config;

pub trait SettingsBox {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn badge<'a>(&self) -> Option<Container<'a, Message>>;
    /// True means it's running, False means it's not, None means
    /// it's not supposed to show the "Running" badge
    fn running(&self) -> Option<bool>;
    fn body<'a>(&self, config: &Config) -> Column<'a, Message>;
    fn display<'a>(&self, config: &Config) -> Container<'a, Message> {
        let mut title_row = Row::new();
        if let Some(badge) = self.badge() {
            title_row = title_row.push(badge);
        }
        title_row = title_row
            .push(
                Row::new()
                    .push(text::bold(text::simple(self.title())))
                    .width(Length::Fill),
            )
            .spacing(10)
            .align_items(iced::Align::Center);

        if let Some(running) = self.running() {
            let running_container = if running {
                Container::new(
                    Row::new()
                        .push(dot_icon().size(5).color(color::SUCCESS))
                        .push(text::small("Running").color(color::SUCCESS))
                        .align_items(iced::Align::Center),
                )
            } else {
                Container::new(
                    Row::new()
                        .push(dot_icon().size(5).color(color::WARNING))
                        .push(text::small("Not running").color(color::WARNING))
                        .align_items(iced::Align::Center),
                )
            };

            title_row = title_row.push(running_container).width(Length::Shrink);
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Container::new(Row::new().push(title_row).spacing(20))
                                .width(Length::Fill),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                )
                .push(separation().width(Length::Fill))
                .push(self.body(config))
                .spacing(20),
        ))
        .width(Length::Fill)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SettingsBoxes {
    pub general: GeneralBox,
    pub manager: ManagerBox,
    pub stakeholder: StakeholderBox,
    pub bitcoin: BitcoinCoreBox,
}

impl SettingsBoxes {
    pub fn new(bitcoin_blockheight: u64) -> Self {
        SettingsBoxes {
            bitcoin: BitcoinCoreBox {
                blockheight: bitcoin_blockheight,
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GeneralBox {}

impl SettingsBox for GeneralBox {
    fn title(&self) -> &'static str {
        "General"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        None
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let rows = vec![
            ("Coordinator host", config.coordinator_host.clone()),
            (
                "Coordinator noise key",
                config.coordinator_noise_key.clone(),
            ),
            (
                "Coordinator poll",
                config
                    .coordinator_poll_seconds
                    .map(|p| format!("{} seconds", p))
                    .unwrap_or_else(|| "Not set".to_string()),
            ),
            (
                "Data dir",
                config
                    .data_dir
                    .clone()
                    .map(|d| format!("{:?}", d))
                    .unwrap_or_else(|| "Not set".to_string()),
            ),
            (
                "Daemon",
                config
                    .daemon
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "Not set".to_string()),
            ),
            (
                "Log level",
                config
                    .log_level
                    .clone()
                    .unwrap_or_else(|| "Not set".to_string()),
            ),
        ];
        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(text::small(k)).width(Length::Fill))
                    .push(text::small(&v)),
            );
        }
        column
    }
}

#[derive(Debug, Clone, Default)]
pub struct StakeholderBox {}

impl SettingsBox for StakeholderBox {
    fn title(&self) -> &'static str {
        "My stakeholder info"
    }

    fn description(&self) -> &'static str {
        "Stakeholder-specific parameters, such as the xpub, the emergency_address, the watchtowers"
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        None
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let config = config.stakeholder_config.as_ref().unwrap();
        let rows = vec![
            ("xpub", config.xpub.to_string()),
            ("Emergency address", config.emergency_address.clone()),
        ];
        let mut general_column = Column::new();
        for (k, v) in rows {
            general_column = general_column.push(
                Row::new()
                    .push(Container::new(text::small(k)).width(Length::Fill))
                    .push(text::small(&v)),
            );
        }

        let mut watchtowers_column = Column::new();
        for w in &config.watchtowers {
            watchtowers_column = watchtowers_column.push(
                Row::new()
                    .push(Container::new(text::small(&w.host)).width(Length::Fill))
                    .push(text::small(&w.noise_key)),
            );
        }

        Column::new()
            .push(general_column)
            .push(separation().width(Length::Fill))
            .push(
                Column::new()
                    .push(Container::new(text::bold(text::small("Watchtowers"))))
                    .push(watchtowers_column)
                    .spacing(8),
            )
            .spacing(20)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManagerBox {}

impl SettingsBox for ManagerBox {
    fn title(&self) -> &'static str {
        "My manager info"
    }

    fn description(&self) -> &'static str {
        "Manager-specific parameters, such as the xpub and the cosigners"
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        None
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let config = config.manager_config.as_ref().unwrap();
        let mut cosigners_column = Column::new();
        for c in &config.cosigners {
            cosigners_column = cosigners_column.push(
                Row::new()
                    .push(Container::new(text::small(&c.host)).width(Length::Fill))
                    .push(text::small(&c.noise_key)),
            );
        }

        Column::new()
            .push(
                Row::new()
                    .push(Container::new(text::small("xpub")).width(Length::Fill))
                    .push(text::small(&config.xpub.to_string())),
            )
            .push(separation().width(Length::Fill))
            .push(
                Column::new()
                    .push(Container::new(text::bold(text::small("Cosigners"))))
                    .push(cosigners_column)
                    .spacing(8),
            )
            .spacing(20)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BitcoinCoreBox {
    blockheight: u64,
}

impl SettingsBox for BitcoinCoreBox {
    fn title(&self) -> &'static str {
        "Bitcoin Core"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        Some(badge::bitcoin_core())
    }

    fn running(&self) -> Option<bool> {
        Some(true)
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let mut col = Column::new().spacing(20);
        if self.blockheight != 0 {
            col =
                col.push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(badge::network())
                                .push(Column::new().push(text::simple("Network:")).push(
                                    text::bold(text::simple(
                                        &config.bitcoind_config.network.to_string(),
                                    )),
                                ))
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        )
                        .push(
                            Row::new()
                                .push(badge::block())
                                .push(
                                    Column::new().push(text::simple("Block Height:")).push(
                                        text::bold(text::simple(&self.blockheight.to_string())),
                                    ),
                                )
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        ),
                );
        }

        let config = &config.bitcoind_config;
        let rows = vec![
            (
                "Cookie file path",
                config.cookie_path.to_str().unwrap().to_string(),
            ),
            ("Socket address", config.addr.to_string()),
            (
                "Poll interval",
                config
                    .poll_interval_secs
                    .map(|p| format!("{} seconds", p))
                    .unwrap_or_else(|| "Not set".to_string()),
            ),
        ];

        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(text::small(k)).width(Length::Fill))
                    .push(text::small(&v)),
            );
        }

        col.push(column)
    }
}
