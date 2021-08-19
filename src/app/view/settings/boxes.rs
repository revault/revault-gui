use iced::{Align, Column, Container, Length, Row};

use crate::{
    app::message::Message,
    ui::{
        color,
        component::{badge, card, separation, text},
        icon::dot_icon,
    },
};

use crate::revaultd::{config::Config, ServerStatusResponse};

pub trait SettingsBox {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn badge<'a>(&self) -> Option<Container<'a, Message>>;
    /// Some(true) means it's running, Some(false) means it's not, None means
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
    pub bitcoin: BitcoinCoreBox,
    pub coordinator: CoordinatorBox,
    pub cosigners: Vec<CosignerBox>,
    pub watchtowers: Vec<WatchtowerBox>,
    pub advanced: AdvancedBox,
}

impl SettingsBoxes {
    pub fn new(bitcoin_blockheight: u64, server_status: ServerStatusResponse) -> Self {
        let mut cosigners: Vec<_> = server_status
            .cosigners
            .iter()
            .map(|c| CosignerBox {
                running: c.reachable,
                host: c.host.clone(),
            })
            .collect();
        cosigners.sort_by(|a, b| a.host.partial_cmp(&b.host).unwrap());

        let mut watchtowers: Vec<_> = server_status
            .watchtowers
            .iter()
            .map(|w| WatchtowerBox {
                running: w.reachable,
                host: w.host.clone(),
            })
            .collect();
        watchtowers.sort_by(|a, b| a.host.partial_cmp(&b.host).unwrap());

        SettingsBoxes {
            bitcoin: BitcoinCoreBox {
                blockheight: bitcoin_blockheight,
            },
            coordinator: CoordinatorBox {
                running: server_status.coordinator.reachable,
            },
            cosigners,
            watchtowers,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdvancedBox {}

impl SettingsBox for AdvancedBox {
    fn title(&self) -> &'static str {
        "Advanced"
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

#[derive(Debug, Clone, Default)]
pub struct CoordinatorBox {
    running: bool,
}

impl SettingsBox for CoordinatorBox {
    fn title(&self) -> &'static str {
        "Coordinator"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        Some(self.running)
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let rows = vec![
            ("Host", config.coordinator_host.clone()),
            ("Noise key", config.coordinator_noise_key.clone()),
            (
                "Poll interval",
                config
                    .coordinator_poll_seconds
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
        column
    }
}

#[derive(Debug, Clone, Default)]
pub struct CosignerBox {
    running: bool,
    host: String,
}

impl SettingsBox for CosignerBox {
    fn title(&self) -> &'static str {
        "Cosigner"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        Some(self.running)
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let cosigner_config = config
            .manager_config
            .as_ref()
            .unwrap()
            .cosigners
            .iter()
            .find(|c| c.host == self.host)
            .unwrap();
        let rows = vec![
            ("Host", cosigner_config.host.clone()),
            ("Noise key", cosigner_config.noise_key.clone()),
            // TODO maybe having the bitcoin public key here?
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
pub struct WatchtowerBox {
    running: bool,
    host: String,
}

impl SettingsBox for WatchtowerBox {
    fn title(&self) -> &'static str {
        "Watchtower"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn badge<'a>(&self) -> Option<Container<'a, Message>> {
        None
    }

    fn running(&self) -> Option<bool> {
        Some(self.running)
    }

    fn body<'a>(&self, config: &Config) -> Column<'a, Message> {
        let watchtower_config = config
            .stakeholder_config
            .as_ref()
            .unwrap()
            .watchtowers
            .iter()
            .find(|c| c.host == self.host)
            .unwrap();
        let rows = vec![
            ("Host", watchtower_config.host.clone()),
            ("Noise key", watchtower_config.noise_key.clone()),
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
