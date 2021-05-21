use iced::{
    button::State as Button, scrollable, text_input, Align, Column, Container, Element, Length,
    Row, TextInput,
};

use crate::{
    installer::message::{self, Message},
    revault::Role,
    ui::{
        color,
        component::{button, image::revault_colored_logo, scroll, text, ContainerBackgroundStyle},
        icon,
    },
};

pub fn welcome(install_button: &mut Button) -> Element<Message> {
    Container::new(Container::new(
        Column::new()
            .push(Container::new(
                revault_colored_logo()
                    .width(Length::Units(400))
                    .height(Length::Fill),
            ))
            .push(
                button::primary(install_button, button::button_content(None, "Install"))
                    .on_press(Message::Next),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Align::Center),
    ))
    .center_y()
    .center_x()
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

pub fn define_role<'a>(
    stakeholder_button: &'a mut Button,
    manager_button: &'a mut Button,
    stakeholder_manager_button: &'a mut Button,
    scroll: &'a mut scrollable::State,
) -> Element<'a, Message> {
    layout_center(
        scroll,
        Column::new()
            .push(
                Row::new()
                    .push(
                        button::white_card_button(
                            stakeholder_button,
                            button::button_content(None, "Stakeholder"),
                        )
                        .on_press(Message::Role(&Role::STAKEHOLDER_ONLY)),
                    )
                    .push(
                        button::white_card_button(
                            stakeholder_manager_button,
                            button::button_content(None, "Stakeholder & Manager"),
                        )
                        .on_press(Message::Role(&Role::STAKEHOLDER_AND_MANAGER)),
                    )
                    .push(
                        button::white_card_button(
                            manager_button,
                            button::button_content(None, "Manager"),
                        )
                        .on_press(Message::Role(&Role::MANAGER_ONLY)),
                    )
                    .spacing(20),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Align::Center)
            .into(),
    )
}

pub fn participant_xpub<'a>(
    xpub: &str,
    xpub_input: &'a mut text_input::State,
    delete_button: &'a mut Button,
    warning: bool,
) -> Element<'a, message::ParticipantXpub> {
    let mut col = Column::new().push(
        Row::new()
            .push(
                TextInput::new(
                    xpub_input,
                    "Xpub",
                    xpub,
                    message::ParticipantXpub::XpubEdited,
                )
                .size(15)
                .padding(10),
            )
            .push(
                button::transparent(delete_button, Container::new(icon::trash_icon()))
                    .on_press(message::ParticipantXpub::Delete),
            )
            .spacing(5)
            .align_items(Align::Center),
    );

    if warning {
        col = col.push(text::simple("Please enter a valid xpub").color(color::WARNING))
    }
    Container::new(col.spacing(10)).into()
}

pub fn cosigner_key<'a>(
    key: &str,
    key_input: &'a mut text_input::State,
    delete_button: &'a mut Button,
    warning: bool,
) -> Element<'a, message::CosignerKey> {
    let mut col = Column::new().push(
        Row::new()
            .push(
                TextInput::new(key_input, "Key", key, message::CosignerKey::KeyEdited)
                    .size(15)
                    .padding(10),
            )
            .push(
                button::transparent(delete_button, Container::new(icon::trash_icon()))
                    .on_press(message::CosignerKey::Delete),
            )
            .spacing(5)
            .align_items(Align::Center),
    );

    if warning {
        col = col.push(text::simple("Please enter a valid key").color(color::WARNING))
    }
    Container::new(col.spacing(10)).into()
}

pub fn define_stakeholder_xpubs_as_stakeholder<'a>(
    our_xpub: &str,
    our_xpub_input: &'a mut text_input::State,
    our_xpub_warning: bool,
    add_xpub_button: &'a mut Button,
    other_xpubs: Vec<Element<'a, Message>>,
    scroll: &'a mut scrollable::State,
    previous_button: &'a mut Button,
    save_button: &'a mut Button,
) -> Element<'a, Message> {
    let mut col = Column::new()
        .push(text::bold(text::simple("Your stakeholder xpub:")))
        .push(
            TextInput::new(our_xpub_input, "Your stakeholder xpub", our_xpub, |msg| {
                Message::DefineStakeholderXpubs(message::DefineStakeholderXpubs::OurXpubEdited(msg))
            })
            .size(15)
            .padding(10),
        )
        .spacing(10);

    if our_xpub_warning {
        col = col.push(text::simple("Please enter a valid xpub").color(color::WARNING));
    }

    layout(
        scroll,
        previous_button,
        Column::new()
            .push(text::bold(text::simple("Define stakeholders")).size(50))
            .push(col)
            .push(
                Column::new()
                    .spacing(10)
                    .push(text::bold(text::simple("Other stakeholders xpubs:")))
                    .push(Column::with_children(other_xpubs).spacing(10))
                    .push(
                        Container::new(
                            button::white_card_button(
                                add_xpub_button,
                                button::button_content(Some(icon::plus_icon()), "Add stakeholder"),
                            )
                            .on_press(Message::DefineStakeholderXpubs(
                                message::DefineStakeholderXpubs::AddXpub,
                            )),
                        )
                        .width(Length::Fill),
                    ),
            )
            .push(
                Row::new()
                    .push(
                        button::primary(save_button, button::button_content(None, "Save"))
                            .on_press(Message::Next),
                    )
                    .align_items(Align::Center)
                    .spacing(20),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Align::Center)
            .into(),
    )
}

pub fn define_stakeholder_xpubs_as_manager_only<'a>(
    add_xpub_button: &'a mut Button,
    stakeholder_xpubs: Vec<Element<'a, Message>>,
    scroll: &'a mut scrollable::State,
    previous_button: &'a mut Button,
    save_button: &'a mut Button,
) -> Element<'a, Message> {
    let mut row = Row::new().align_items(Align::Center).spacing(20);
    if stakeholder_xpubs.is_empty() {
        row = row.push(button::primary(
            save_button,
            button::button_content(None, "Save"),
        ));
    } else {
        row = row.push(
            button::primary(save_button, button::button_content(None, "Save"))
                .on_press(Message::Next),
        );
    }

    layout(
        scroll,
        previous_button,
        Column::new()
            .push(text::bold(text::simple("Define stakeholders")).size(50))
            .push(
                Column::new()
                    .spacing(10)
                    .push(text::bold(text::simple("Stakeholders xpubs:")))
                    .push(Column::with_children(stakeholder_xpubs).spacing(10))
                    .push(
                        Container::new(
                            button::white_card_button(
                                add_xpub_button,
                                button::button_content(Some(icon::plus_icon()), "Add stakeholder"),
                            )
                            .on_press(Message::DefineStakeholderXpubs(
                                message::DefineStakeholderXpubs::AddXpub,
                            )),
                        )
                        .width(Length::Fill),
                    ),
            )
            .push(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Align::Center)
            .into(),
    )
}

pub struct ManagersTreshold {
    increment_button: Button,
    decrement_button: Button,
}

impl ManagersTreshold {
    pub fn new() -> Self {
        Self {
            increment_button: Button::new(),
            decrement_button: Button::new(),
        }
    }

    pub fn render(&mut self, managers_treshold: u32) -> Container<Message> {
        Container::new(
            Column::new()
                .push(text::bold(text::simple("Managers treshold:")))
                .push(
                    Row::new()
                        .push(text::simple(&format!("{}", managers_treshold)).size(50))
                        .push(
                            Column::new()
                                .push(
                                    button::transparent(
                                        &mut self.increment_button,
                                        Container::new(text::simple("+")),
                                    )
                                    .on_press(
                                        Message::DefineManagerXpubs(
                                            message::DefineManagerXpubs::ManagersTreshold(
                                                message::Action::Increment,
                                            ),
                                        ),
                                    ),
                                )
                                .push(
                                    button::transparent(
                                        &mut self.decrement_button,
                                        Container::new(text::simple("-")),
                                    )
                                    .on_press(
                                        Message::DefineManagerXpubs(
                                            message::DefineManagerXpubs::ManagersTreshold(
                                                message::Action::Decrement,
                                            ),
                                        ),
                                    ),
                                )
                                .align_items(Align::Center),
                        )
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .align_items(Align::Center)
                .spacing(10),
        )
    }
}

pub struct SpendingDelay {
    increment_button: Button,
    decrement_button: Button,
}

impl SpendingDelay {
    pub fn new() -> Self {
        Self {
            increment_button: Button::new(),
            decrement_button: Button::new(),
        }
    }

    pub fn render(&mut self, spending_delay: u32) -> Container<Message> {
        Container::new(
            Column::new()
                .push(text::bold(text::simple("Spending delay in blocks:")))
                .push(
                    Row::new()
                        .push(text::simple(&format!("{}", spending_delay)).size(50))
                        .push(
                            Column::new()
                                .push(
                                    button::transparent(
                                        &mut self.increment_button,
                                        Container::new(text::simple("+")),
                                    )
                                    .on_press(
                                        Message::DefineManagerXpubs(
                                            message::DefineManagerXpubs::SpendingDelay(
                                                message::Action::Increment,
                                            ),
                                        ),
                                    ),
                                )
                                .push(
                                    button::transparent(
                                        &mut self.decrement_button,
                                        Container::new(text::simple("-")),
                                    )
                                    .on_press(
                                        Message::DefineManagerXpubs(
                                            message::DefineManagerXpubs::SpendingDelay(
                                                message::Action::Decrement,
                                            ),
                                        ),
                                    ),
                                )
                                .align_items(Align::Center),
                        )
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .align_items(Align::Center)
                .spacing(10),
        )
    }
}

pub struct DefineManagerXpubsAsManager {
    managers_treshold: ManagersTreshold,
    spending_delay: SpendingDelay,
    add_xpub_button: Button,
    add_cosigner_button: Button,
    our_xpub_input: text_input::State,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineManagerXpubsAsManager {
    pub fn new() -> Self {
        Self {
            our_xpub_input: text_input::State::new(),
            add_xpub_button: Button::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
            add_cosigner_button: Button::new(),
            spending_delay: SpendingDelay::new(),
            managers_treshold: ManagersTreshold::new(),
        }
    }

    pub fn render<'a>(
        &'a mut self,
        managers_treshold: u32,
        spending_delay: u32,
        our_xpub: &str,
        our_xpub_warning: bool,
        other_xpubs: Vec<Element<'a, Message>>,
        cosigners: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut manager_xpub_col = Column::new()
            .push(text::bold(text::simple("Your manager xpub:")))
            .push(
                TextInput::new(
                    &mut self.our_xpub_input,
                    "Your manager xpub",
                    our_xpub,
                    |msg| {
                        Message::DefineManagerXpubs(message::DefineManagerXpubs::OurXpubEdited(msg))
                    },
                )
                .size(15)
                .padding(10),
            )
            .spacing(10);

        if our_xpub_warning {
            manager_xpub_col = manager_xpub_col
                .push(text::simple("Please enter a valid xpub").color(color::WARNING));
        }

        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define managers")).size(50))
                .push(
                    Row::new()
                        .push(
                            Container::new(self.managers_treshold.render(managers_treshold))
                                .width(Length::FillPortion(1))
                                .align_x(Align::Center),
                        )
                        .push(
                            Container::new(self.spending_delay.render(spending_delay))
                                .width(Length::FillPortion(1))
                                .align_x(Align::Center),
                        )
                        .width(Length::Fill),
                )
                .push(manager_xpub_col)
                .push(
                    Column::new()
                        .push(text::bold(text::simple("Other Managers xpubs:")))
                        .push(Column::with_children(other_xpubs).spacing(10))
                        .push(
                            Container::new(
                                button::white_card_button(
                                    &mut self.add_xpub_button,
                                    button::button_content(Some(icon::plus_icon()), "Add manager"),
                                )
                                .on_press(
                                    Message::DefineManagerXpubs(
                                        message::DefineManagerXpubs::AddXpub,
                                    ),
                                ),
                            )
                            .width(Length::Fill),
                        )
                        .spacing(10),
                )
                .push(
                    Column::new()
                        .push(text::bold(text::simple("Cosigners keys:")))
                        .push(Column::with_children(cosigners).spacing(10))
                        .push(
                            Container::new(
                                button::white_card_button(
                                    &mut self.add_cosigner_button,
                                    button::button_content(
                                        Some(icon::plus_icon()),
                                        "Add cosigner key",
                                    ),
                                )
                                .on_press(
                                    Message::DefineManagerXpubs(
                                        message::DefineManagerXpubs::AddCosigner,
                                    ),
                                ),
                            )
                            .width(Length::Fill),
                        )
                        .spacing(10),
                )
                .push(
                    Row::new()
                        .push(
                            button::primary(
                                &mut self.save_button,
                                button::button_content(None, "Save"),
                            )
                            .on_press(Message::Next),
                        )
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

pub struct DefineManagerXpubsAsStakeholderOnly {
    managers_treshold: ManagersTreshold,
    spending_delay: SpendingDelay,
    add_cosigner_button: Button,
    add_xpub_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineManagerXpubsAsStakeholderOnly {
    pub fn new() -> Self {
        Self {
            managers_treshold: ManagersTreshold::new(),
            spending_delay: SpendingDelay::new(),
            add_cosigner_button: Button::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
            add_xpub_button: Button::new(),
        }
    }
    pub fn render<'a>(
        &'a mut self,
        managers_treshold: u32,
        spending_delay: u32,
        manager_xpubs: Vec<Element<'a, Message>>,
        cosigners: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut row = Row::new().align_items(Align::Center).spacing(20);
        if manager_xpubs.is_empty() {
            row = row.push(button::primary(
                &mut self.save_button,
                button::button_content(None, "Save"),
            ));
        } else {
            row = row.push(
                button::primary(&mut self.save_button, button::button_content(None, "Save"))
                    .on_press(Message::Next),
            );
        }

        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define managers")).size(50))
                .push(
                    Row::new()
                        .push(
                            Container::new(self.managers_treshold.render(managers_treshold))
                                .width(Length::FillPortion(1))
                                .align_x(Align::Center),
                        )
                        .push(
                            Container::new(self.spending_delay.render(spending_delay))
                                .width(Length::FillPortion(1))
                                .align_x(Align::Center),
                        )
                        .width(Length::Fill),
                )
                .push(
                    Column::new()
                        .spacing(10)
                        .push(text::bold(text::simple("Managers xpubs:")))
                        .push(Column::with_children(manager_xpubs).spacing(10))
                        .push(
                            Container::new(
                                button::white_card_button(
                                    &mut self.add_xpub_button,
                                    button::button_content(Some(icon::plus_icon()), "Add manager"),
                                )
                                .on_press(
                                    Message::DefineManagerXpubs(
                                        message::DefineManagerXpubs::AddXpub,
                                    ),
                                ),
                            )
                            .width(Length::Fill),
                        ),
                )
                .push(
                    Column::new()
                        .spacing(10)
                        .push(text::bold(text::simple("Cosigners keys:")))
                        .push(Column::with_children(cosigners).spacing(10))
                        .push(
                            Container::new(
                                button::white_card_button(
                                    &mut self.add_cosigner_button,
                                    button::button_content(Some(icon::plus_icon()), "Add cosigner"),
                                )
                                .on_press(
                                    Message::DefineManagerXpubs(
                                        message::DefineManagerXpubs::AddCosigner,
                                    ),
                                ),
                            )
                            .width(Length::Fill),
                        ),
                )
                .push(row)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

pub fn define_cpfp_descriptor<'a>(
    add_xpub_button: &'a mut Button,
    manager_xpubs: Vec<Element<'a, Message>>,
    scroll: &'a mut scrollable::State,
    previous_button: &'a mut Button,
    save_button: &'a mut Button,
) -> Element<'a, Message> {
    let mut row = Row::new().align_items(Align::Center).spacing(20);
    if manager_xpubs.is_empty() {
        row = row.push(button::primary(
            save_button,
            button::button_content(None, "Save"),
        ));
    } else {
        row = row.push(
            button::primary(save_button, button::button_content(None, "Save"))
                .on_press(Message::Next),
        );
    }

    layout(
        scroll,
        previous_button,
        Column::new()
            .push(text::bold(text::simple("Define fee bumping managers")).size(50))
            .push(
                Column::new()
                    .spacing(10)
                    .push(text::bold(text::simple("Managers xpubs:")))
                    .push(Column::with_children(manager_xpubs).spacing(10))
                    .push(
                        Container::new(
                            button::white_card_button(
                                add_xpub_button,
                                button::button_content(Some(icon::plus_icon()), "Add manager"),
                            )
                            .on_press(Message::DefineCpfpDescriptor(
                                message::DefineCpfpDescriptor::AddXpub,
                            )),
                        )
                        .width(Length::Fill),
                    ),
            )
            .push(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Align::Center)
            .into(),
    )
}

pub struct DefineCoordinator {
    host_input: text_input::State,
    noise_key_input: text_input::State,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineCoordinator {
    pub fn new() -> Self {
        Self {
            host_input: text_input::State::new(),
            noise_key_input: text_input::State::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
    pub fn render<'a>(
        &'a mut self,
        host: &str,
        noise_key: &str,
        warning: bool,
    ) -> Element<'a, Message> {
        let mut row = Row::new().align_items(Align::Center).spacing(20);
        if warning {
            row = row.push(button::primary(
                &mut self.save_button,
                button::button_content(None, "Save"),
            ));
        } else {
            row = row.push(
                button::primary(&mut self.save_button, button::button_content(None, "Save"))
                    .on_press(Message::Next),
            );
        }

        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define coordinator")).size(50))
                .push(
                    Column::new()
                        .push(text::bold(text::simple("Host:")))
                        .push(
                            TextInput::new(&mut self.host_input, "Host", host, |msg| {
                                Message::DefineCoordinator(message::DefineCoordinator::HostEdited(
                                    msg,
                                ))
                            })
                            .size(15)
                            .padding(10),
                        )
                        .spacing(10),
                )
                .push(
                    Column::new()
                        .push(text::bold(text::simple("Noise key:")))
                        .push(
                            TextInput::new(
                                &mut self.noise_key_input,
                                "Noise key",
                                noise_key,
                                |msg| {
                                    Message::DefineCoordinator(
                                        message::DefineCoordinator::NoiseKeyEdited(msg),
                                    )
                                },
                            )
                            .size(15)
                            .padding(10),
                        )
                        .spacing(10),
                )
                .push(row)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

pub struct DefineEmergencyAddress {
    address_input: text_input::State,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineEmergencyAddress {
    pub fn new() -> Self {
        Self {
            address_input: text_input::State::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
    pub fn render<'a>(&'a mut self, address: &str, warning: bool) -> Element<'a, Message> {
        let mut row = Row::new().align_items(Align::Center).spacing(20);
        if warning {
            row = row.push(button::primary(
                &mut self.save_button,
                button::button_content(None, "Save"),
            ));
        } else {
            row = row.push(
                button::primary(&mut self.save_button, button::button_content(None, "Save"))
                    .on_press(Message::Next),
            );
        }
        let mut col = Column::new()
            .push(text::bold(text::simple("address:")))
            .push(
                TextInput::new(
                    &mut self.address_input,
                    "address",
                    address,
                    Message::DefineEmergencyAddress,
                )
                .size(15)
                .padding(10),
            )
            .spacing(10);

        if warning {
            col = col.push(text::simple("Please enter a valid address").color(color::WARNING))
        }

        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define emergency address")).size(50))
                .push(col)
                .push(row)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

pub struct Watchtower {
    noise_key_input: text_input::State,
    host_input: text_input::State,
    delete_button: Button,
}

impl Watchtower {
    pub fn new() -> Self {
        Self {
            noise_key_input: text_input::State::new(),
            host_input: text_input::State::new(),
            delete_button: Button::new(),
        }
    }
    pub fn render(
        &mut self,
        host: &str,
        noise_key: &str,
        warning_host: bool,
        warning_noise_key: bool,
    ) -> Element<message::DefineWatchtower> {
        let mut col = Column::new().push(
            Row::new()
                .push(
                    TextInput::new(
                        &mut self.host_input,
                        "Host",
                        host,
                        message::DefineWatchtower::HostEdited,
                    )
                    .size(15)
                    .padding(10),
                )
                .push(
                    TextInput::new(
                        &mut self.noise_key_input,
                        "Noise key",
                        noise_key,
                        message::DefineWatchtower::NoiseKeyEdited,
                    )
                    .size(15)
                    .padding(10),
                )
                .push(
                    button::transparent(
                        &mut self.delete_button,
                        Container::new(icon::trash_icon()),
                    )
                    .on_press(message::DefineWatchtower::Delete),
                )
                .spacing(5)
                .align_items(Align::Center),
        );

        if warning_host {
            col = col.push(text::simple("Please enter a valid host").color(color::WARNING))
        }
        if warning_noise_key {
            col = col.push(text::simple("Please enter a valid noise_key").color(color::WARNING))
        }
        Container::new(col.spacing(10)).into()
    }
}

pub struct DefineWatchtowers {
    add_watchtower_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineWatchtowers {
    pub fn new() -> Self {
        Self {
            add_watchtower_button: Button::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
    pub fn render<'a>(
        &'a mut self,
        watchtowers: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define your watchtowers")).size(50))
                .push(
                    Column::new()
                        .push(
                            Container::new(text::bold(text::simple("Your watchtowers:")))
                                .width(Length::Fill),
                        )
                        .push(Column::with_children(watchtowers).spacing(10))
                        .push(
                            button::transparent(
                                &mut self.add_watchtower_button,
                                button::button_content(Some(icon::plus_icon()), "Add a watchtower"),
                            )
                            .on_press(Message::DefineWatchtowers(
                                message::DefineWatchtowers::AddWatchtower,
                            )),
                        )
                        .spacing(10),
                )
                .push(
                    button::primary(&mut self.save_button, button::button_content(None, "Save"))
                        .on_press(Message::Next),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

pub struct Cosigner {
    noise_key_input: text_input::State,
    host_input: text_input::State,
}

impl Cosigner {
    pub fn new() -> Self {
        Self {
            noise_key_input: text_input::State::new(),
            host_input: text_input::State::new(),
        }
    }
    pub fn render(
        &mut self,
        host: &str,
        noise_key: &str,
        warning_host: bool,
        warning_noise_key: bool,
    ) -> Element<message::DefineCosigner> {
        let mut col = Column::new().push(
            Row::new()
                .push(
                    TextInput::new(
                        &mut self.host_input,
                        "Host",
                        host,
                        message::DefineCosigner::HostEdited,
                    )
                    .size(15)
                    .padding(10),
                )
                .push(
                    TextInput::new(
                        &mut self.noise_key_input,
                        "Noise key",
                        noise_key,
                        message::DefineCosigner::NoiseKeyEdited,
                    )
                    .size(15)
                    .padding(10),
                )
                .spacing(5)
                .align_items(Align::Center),
        );

        if warning_host {
            col = col.push(text::simple("Please enter a valid host").color(color::WARNING))
        }
        if warning_noise_key {
            col = col.push(text::simple("Please enter a valid noise_key").color(color::WARNING))
        }
        Container::new(col.spacing(10)).into()
    }
}

pub struct DefineCosigners {
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineCosigners {
    pub fn new() -> Self {
        Self {
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
    pub fn render<'a>(
        &'a mut self,
        watchtowers: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        layout(
            &mut self.scroll,
            &mut self.previous_button,
            Column::new()
                .push(text::bold(text::simple("Define the cosigners")).size(50))
                .push(
                    Column::new()
                        .push(
                            Container::new(text::bold(text::simple("The cosigners:")))
                                .width(Length::Fill),
                        )
                        .push(Column::with_children(watchtowers).spacing(10))
                        .spacing(10),
                )
                .push(
                    button::primary(&mut self.save_button, button::button_content(None, "Save"))
                        .on_press(Message::Next),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(100)
                .spacing(50)
                .align_items(Align::Center)
                .into(),
        )
    }
}

fn layout<'a>(
    scroll_state: &'a mut scrollable::State,
    previous_button: &'a mut Button,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    Container::new(scroll(
        scroll_state,
        Container::new(
            Column::new()
                .push(
                    button::transparent(
                        previous_button,
                        button::button_content(None, "< Previous"),
                    )
                    .on_press(Message::Previous),
                )
                .push(Container::new(content).width(Length::Fill).center_x()),
        ),
    ))
    .style(ContainerBackgroundStyle)
    .center_x()
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn layout_center<'a>(
    scroll_state: &'a mut scrollable::State,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    Container::new(scroll(scroll_state, Container::new(content)))
        .style(ContainerBackgroundStyle)
        .center_y()
        .center_x()
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}
