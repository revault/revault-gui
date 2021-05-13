use iced::{Align, Column, Container, Length, Row};

use crate::app::{
    component::{card, separation, text},
    message::Message,
};

use crate::revaultd::config::Config;

pub trait SettingsBox {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn body<'a>(&self, config: &Config) -> Column<'a, Message>;
    fn display<'a>(&self, config: &Config) -> Container<'a, Message> {
        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(
                                        Column::new()
                                            .push(text::bold(text::simple(self.title())))
                                            .push(text::small(self.description())),
                                    )
                                    .spacing(20),
                            )
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
    pub bitcoind: BitcoindBox,
    pub scripts: ScriptsBox,
    pub manager: ManagerBox,
    pub stakeholder: StakeholderBox,
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
                    .unwrap_or("Not set".to_string()),
            ),
            (
                "Data dir",
                config
                    .data_dir
                    .clone()
                    .map(|d| format!("{:?}", d))
                    .unwrap_or("Not set".to_string()),
            ),
            (
                "Daemon",
                config
                    .daemon
                    .map(|d| d.to_string())
                    .unwrap_or("Not set".to_string()),
            ),
            (
                "Log level",
                config
                    .log_level
                    .clone()
                    .map(|d| d.to_string())
                    .unwrap_or("Not set".to_string()),
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
pub struct BitcoindBox {}

impl SettingsBox for BitcoindBox {
    fn title(&self) -> &'static str {
        "Bitcoind"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let config = &config.bitcoind_config;
        let rows = vec![
            ("Network", config.network.to_string()),
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
                    .unwrap_or("Not set".to_string()),
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
pub struct ScriptsBox {}

impl SettingsBox for ScriptsBox {
    fn title(&self) -> &'static str {
        "Bitcoin scripts"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        Column::new()
            .push(
                Column::new()
                    .spacing(5)
                    .push(text::bold(text::small("Deposit descriptor")))
                    .push(text::small(&config.scripts_config.deposit_descriptor)),
            )
            .push(
                Column::new()
                    .spacing(5)
                    .push(text::bold(text::small("Unvault descriptor")))
                    .push(text::small(&config.scripts_config.unvault_descriptor)),
            )
            .push(
                Column::new()
                    .spacing(5)
                    .push(text::bold(text::small("CPFP descriptor")))
                    .push(text::small(&config.scripts_config.cpfp_descriptor)),
            )
            .spacing(10)
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
