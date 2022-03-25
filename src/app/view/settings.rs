use iced::{text_input, Align, Column, Container, Element, Length, Row};

use revault_ui::{
    color,
    component::{badge, button, card, form, separation, text::Text},
    icon,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, SettingsMessage},
        view::layout,
    },
    revault::Role,
};

#[derive(Debug)]
pub struct SettingsView {
    dashboard: layout::Dashboard,
    add_watchtower_button: iced::button::State,
}

impl SettingsView {
    pub fn new() -> Self {
        SettingsView {
            dashboard: layout::Dashboard::new(),
            add_watchtower_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        can_edit: bool,
        settings: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut col = Column::with_children(settings).spacing(20);
        if can_edit && ctx.role == Role::Stakeholder {
            col = col.push(
                Container::new(
                    button::important(
                        &mut self.add_watchtower_button,
                        button::button_content(Some(icon::plus_icon()), "Add watchtower"),
                    )
                    .on_press(Message::AddWatchtower),
                )
                .width(Length::Fill)
                .align_x(Align::End),
            );
        }
        self.dashboard.view(ctx, warning, col)
    }
}

#[derive(Debug, Default)]
pub struct BitcoindSettingsEditView {
    cancel_button: iced::button::State,
    confirm_button: iced::button::State,

    addr_input: text_input::State,
    cookie_path_input: text_input::State,
}

impl BitcoindSettingsEditView {
    pub fn view<'a>(
        &'a mut self,
        config: &revaultd::config::BitcoindConfig,
        blockheight: i32,
        addr: &form::Value<String>,
        cookie_path: &form::Value<String>,
        processing: bool,
    ) -> Element<'a, SettingsMessage> {
        let mut col = Column::new().spacing(20);
        if blockheight != 0 {
            col = col
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(badge::network())
                                .push(
                                    Column::new()
                                        .push(Text::new("Network:"))
                                        .push(Text::new(&config.network.to_string()).bold()),
                                )
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        )
                        .push(
                            Row::new()
                                .push(badge::block())
                                .push(
                                    Column::new()
                                        .push(Text::new("Block Height:"))
                                        .push(Text::new(&blockheight.to_string()).bold()),
                                )
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        ),
                )
                .push(separation().width(Length::Fill));
        }

        col = col
            .push(
                Column::new()
                    .push(Text::new("Cookie file path:").bold().small())
                    .push(
                        form::Form::new(
                            &mut self.cookie_path_input,
                            "Cookie file path",
                            cookie_path,
                            |value| SettingsMessage::FieldEdited("cookie_file_path", value),
                        )
                        .warning("Please enter a valid filesystem path")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(Text::new("Socket address:").bold().small())
                    .push(
                        form::Form::new(&mut self.addr_input, "Socket address:", addr, |value| {
                            SettingsMessage::FieldEdited("socket_address", value)
                        })
                        .warning("Please enter a valid address")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            );

        let mut cancel_button = button::cancel(
            &mut self.cancel_button,
            Container::new(Text::new(" Cancel ")).padding(5),
        );
        let mut confirm_button = button::primary(
            &mut self.confirm_button,
            Container::new(Text::new(" Save ")).padding(5),
        );
        if !processing {
            cancel_button = cancel_button.on_press(SettingsMessage::CancelEdit);
            confirm_button = confirm_button.on_press(SettingsMessage::ConfirmEdit);
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(badge::bitcoin_core())
                        .push(Text::new("Bitcoind"))
                        .padding(10)
                        .spacing(20)
                        .align_items(Align::Center)
                        .width(Length::Fill),
                )
                .push(separation().width(Length::Fill))
                .push(col)
                .push(
                    Container::new(
                        Row::new()
                            .push(cancel_button)
                            .push(confirm_button)
                            .spacing(10)
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct BitcoindSettingsView {
    edit_button: iced::button::State,
}

impl BitcoindSettingsView {
    pub fn view<'a>(
        &'a mut self,
        config: &revaultd::config::BitcoindConfig,
        blockheight: i32,
        is_running: Option<bool>,
        can_edit: bool,
    ) -> Element<'a, SettingsMessage> {
        let mut col = Column::new().spacing(20);
        if blockheight != 0 {
            col = col
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(badge::network())
                                .push(
                                    Column::new()
                                        .push(Text::new("Network:"))
                                        .push(Text::new(&config.network.to_string()).bold()),
                                )
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        )
                        .push(
                            Row::new()
                                .push(badge::block())
                                .push(
                                    Column::new()
                                        .push(Text::new("Block Height:"))
                                        .push(Text::new(&blockheight.to_string()).bold()),
                                )
                                .spacing(10)
                                .width(Length::FillPortion(1)),
                        ),
                )
                .push(separation().width(Length::Fill));
        }

        let rows = vec![
            (
                "Cookie file path:",
                config.cookie_path.to_str().unwrap().to_string(),
            ),
            ("Socket address:", config.addr.to_string()),
        ];

        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(Text::new(k).bold().small()).width(Length::Fill))
                    .push(Text::new(&v).small()),
            );
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(badge::bitcoin_core())
                                .push(Text::new("Bitcoind"))
                                .push(is_running_label(is_running))
                                .spacing(20)
                                .align_items(Align::Center)
                                .width(Length::Fill),
                        )
                        .push(if can_edit {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                            .on_press(SettingsMessage::Edit)
                        } else {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                        })
                        .align_items(Align::Center),
                )
                .push(separation().width(Length::Fill))
                .push(col.push(column))
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct CoordinatorSettingsEditView {
    cancel_button: iced::button::State,
    confirm_button: iced::button::State,

    host_input: text_input::State,
    key_input: text_input::State,
}

impl CoordinatorSettingsEditView {
    pub fn view<'a>(
        &'a mut self,
        host: &form::Value<String>,
        key: &form::Value<String>,
        processing: bool,
    ) -> Element<'a, SettingsMessage> {
        let mut col = Column::new().spacing(20);
        col = col
            .push(
                Column::new()
                    .push(Text::new("Host:").bold().small())
                    .push(
                        form::Form::new(&mut self.host_input, "Host", host, |value| {
                            SettingsMessage::FieldEdited("host", value)
                        })
                        .warning("Please enter a valid socket address")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(Text::new("Public key:").bold().small())
                    .push(
                        form::Form::new(&mut self.key_input, "Key", key, |value| {
                            SettingsMessage::FieldEdited("key", value)
                        })
                        .warning("Please enter a valid public noise key")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            );

        let mut cancel_button = button::cancel(
            &mut self.cancel_button,
            Container::new(Text::new(" Cancel ")).padding(5),
        );
        let mut confirm_button = button::primary(
            &mut self.confirm_button,
            Container::new(Text::new(" Save ")).padding(5),
        );
        if !processing {
            cancel_button = cancel_button.on_press(SettingsMessage::CancelEdit);
            confirm_button = confirm_button.on_press(SettingsMessage::ConfirmEdit);
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new("Coordinator"))
                        .padding(10)
                        .spacing(20)
                        .align_items(Align::Center)
                        .width(Length::Fill),
                )
                .push(separation().width(Length::Fill))
                .push(col)
                .push(
                    Container::new(
                        Row::new()
                            .push(cancel_button)
                            .push(confirm_button)
                            .spacing(10)
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct CoordinatorSettingsView {
    edit_button: iced::button::State,
}

impl CoordinatorSettingsView {
    pub fn view<'a>(
        &'a mut self,
        host: &str,
        key: &str,
        is_running: Option<bool>,
        can_edit: bool,
    ) -> Element<'a, SettingsMessage> {
        let rows = vec![("Host:", host), ("Public key:", key)];

        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(Text::new(k).bold().small()).width(Length::Fill))
                    .push(Text::new(&v).small()),
            );
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(Text::new("Coordinator"))
                                .push(is_running_label(is_running))
                                .spacing(20)
                                .align_items(Align::Center)
                                .width(Length::Fill),
                        )
                        .push(if can_edit {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                            .on_press(SettingsMessage::Edit)
                        } else {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                        })
                        .align_items(Align::Center),
                )
                .push(separation().width(Length::Fill))
                .push(column)
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct WatchtowerSettingsEditView {
    cancel_button: iced::button::State,
    confirm_button: iced::button::State,
    remove_button: iced::button::State,

    host_input: text_input::State,
    key_input: text_input::State,
}

impl WatchtowerSettingsEditView {
    pub fn view<'a>(
        &'a mut self,
        not_saved: bool,
        host: &form::Value<String>,
        key: &form::Value<String>,
        processing: bool,
    ) -> Element<'a, SettingsMessage> {
        let mut col = Column::new().spacing(20);
        col = col
            .push(
                Column::new()
                    .push(Text::new("Host:").bold().small())
                    .push(
                        form::Form::new(&mut self.host_input, "Host", host, |value| {
                            SettingsMessage::FieldEdited("host", value)
                        })
                        .warning("Please enter a valid socket address")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(Text::new("Public key:").bold().small())
                    .push(
                        form::Form::new(&mut self.key_input, "Key", key, |value| {
                            SettingsMessage::FieldEdited("key", value)
                        })
                        .warning("Please enter a valid public noise key")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            );

        let mut cancel_button = button::cancel(
            &mut self.cancel_button,
            Container::new(Text::new(" Cancel ")).padding(5),
        );
        let mut confirm_button = button::primary(
            &mut self.confirm_button,
            Container::new(Text::new(" Save ")).padding(5),
        );
        if !processing {
            if not_saved {
                cancel_button = cancel_button.on_press(SettingsMessage::Remove);
            } else {
                cancel_button = cancel_button.on_press(SettingsMessage::CancelEdit);
            }
            confirm_button = confirm_button.on_press(SettingsMessage::ConfirmEdit);
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new("Watchtower"))
                        .padding(10)
                        .spacing(20)
                        .align_items(Align::Center)
                        .width(Length::Fill),
                )
                .push(separation().width(Length::Fill))
                .push(col)
                .push(
                    Container::new(
                        Row::new()
                            .push(
                                Row::new()
                                    .push(
                                        button::transparent(
                                            &mut self.remove_button,
                                            Container::new(
                                                Row::new()
                                                    .push(icon::trash_icon().color(color::ALERT))
                                                    .push(Text::new("Remove").color(color::ALERT))
                                                    .spacing(10)
                                                    .align_items(Align::Center),
                                            ),
                                        )
                                        .on_press(SettingsMessage::Remove),
                                    )
                                    .width(Length::Fill),
                            )
                            .push(
                                Row::new()
                                    .push(cancel_button)
                                    .push(confirm_button)
                                    .align_items(Align::Center)
                                    .spacing(10),
                            )
                            .align_items(Align::Center),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct WatchtowerSettingsView {
    edit_button: iced::button::State,
}

impl WatchtowerSettingsView {
    pub fn view<'a>(
        &'a mut self,
        host: &str,
        key: &str,
        is_running: Option<bool>,
        can_edit: bool,
    ) -> Element<'a, SettingsMessage> {
        let rows = vec![("Host:", host), ("Public key:", key)];

        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(Text::new(k).bold().small()).width(Length::Fill))
                    .push(Text::new(&v).small()),
            );
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(Text::new("Watchtower"))
                                .push(is_running_label(is_running))
                                .spacing(20)
                                .align_items(Align::Center)
                                .width(Length::Fill),
                        )
                        .push(if can_edit {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                            .on_press(SettingsMessage::Edit)
                        } else {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                        })
                        .align_items(Align::Center),
                )
                .push(separation().width(Length::Fill))
                .push(column)
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct CosignerSettingsEditView {
    cancel_button: iced::button::State,
    confirm_button: iced::button::State,

    host_input: text_input::State,
    key_input: text_input::State,
}

impl CosignerSettingsEditView {
    pub fn view<'a>(
        &'a mut self,
        host: &form::Value<String>,
        key: &form::Value<String>,
        processing: bool,
    ) -> Element<'a, SettingsMessage> {
        let mut col = Column::new().spacing(20);
        col = col
            .push(
                Column::new()
                    .push(Text::new("Host:").bold().small())
                    .push(
                        form::Form::new(&mut self.host_input, "Host", host, |value| {
                            SettingsMessage::FieldEdited("host", value)
                        })
                        .warning("Please enter a valid socket address")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(Text::new("Public key:").bold().small())
                    .push(
                        form::Form::new(&mut self.key_input, "Key", key, |value| {
                            SettingsMessage::FieldEdited("key", value)
                        })
                        .warning("Please enter a valid public noise key")
                        .size(20)
                        .padding(5)
                        .render(),
                    )
                    .spacing(5),
            );

        let mut cancel_button = button::cancel(
            &mut self.cancel_button,
            Container::new(Text::new(" Cancel ")).padding(5),
        );
        let mut confirm_button = button::primary(
            &mut self.confirm_button,
            Container::new(Text::new(" Save ")).padding(5),
        );
        if !processing {
            cancel_button = cancel_button.on_press(SettingsMessage::CancelEdit);
            confirm_button = confirm_button.on_press(SettingsMessage::ConfirmEdit);
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new("Cosigner"))
                        .padding(10)
                        .spacing(20)
                        .align_items(Align::Center)
                        .width(Length::Fill),
                )
                .push(separation().width(Length::Fill))
                .push(col)
                .push(
                    Container::new(
                        Row::new()
                            .push(cancel_button)
                            .push(confirm_button)
                            .align_items(Align::Center)
                            .spacing(10),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Default)]
pub struct CosignerSettingsView {
    edit_button: iced::button::State,
}

impl CosignerSettingsView {
    pub fn view<'a>(
        &'a mut self,
        host: &str,
        key: &str,
        is_running: Option<bool>,
        can_edit: bool,
    ) -> Element<'a, SettingsMessage> {
        let rows = vec![("Host:", host), ("Public key:", key)];

        let mut column = Column::new();
        for (k, v) in rows {
            column = column.push(
                Row::new()
                    .push(Container::new(Text::new(k).bold().small()).width(Length::Fill))
                    .push(Text::new(&v).small()),
            );
        }

        card::simple(Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(
                            Row::new()
                                .push(Text::new("Cosigner"))
                                .push(is_running_label(is_running))
                                .spacing(20)
                                .align_items(Align::Center)
                                .width(Length::Fill),
                        )
                        .push(if can_edit {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                            .on_press(SettingsMessage::Edit)
                        } else {
                            button::white_card_button(
                                &mut self.edit_button,
                                Container::new(icon::pencil_icon()),
                            )
                        })
                        .align_items(Align::Center),
                )
                .push(separation().width(Length::Fill))
                .push(column)
                .spacing(20),
        ))
        .width(Length::Fill)
        .into()
    }
}

pub fn is_running_label<'a, T: 'a>(is_running: Option<bool>) -> Container<'a, T> {
    if let Some(running) = is_running {
        if running {
            Container::new(
                Row::new()
                    .push(icon::dot_icon().size(5).color(color::SUCCESS))
                    .push(Text::new("Running").small().color(color::SUCCESS))
                    .align_items(iced::Align::Center),
            )
        } else {
            Container::new(
                Row::new()
                    .push(icon::dot_icon().size(5).color(color::ALERT))
                    .push(Text::new("Not running").small().color(color::ALERT))
                    .align_items(iced::Align::Center),
            )
        }
    } else {
        Container::new(Column::new())
    }
}
