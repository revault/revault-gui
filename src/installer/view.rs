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

pub fn define_manager_xpubs_as_manager<'a>(
    our_xpub: &str,
    our_xpub_input: &'a mut text_input::State,
    our_xpub_warning: bool,
    add_xpub_button: &'a mut Button,
    other_xpubs: Vec<Element<'a, Message>>,
    add_cosigner_button: &'a mut Button,
    cosigners: Vec<Element<'a, Message>>,
    scroll: &'a mut scrollable::State,
    previous_button: &'a mut Button,
    save_button: &'a mut Button,
) -> Element<'a, Message> {
    let mut col = Column::new()
        .push(text::bold(text::simple("Your manager xpub:")))
        .push(
            TextInput::new(our_xpub_input, "Your manager xpub", our_xpub, |msg| {
                Message::DefineManagerXpubs(message::DefineManagerXpubs::OurXpubEdited(msg))
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
            .push(text::bold(text::simple("Define managers")).size(50))
            .push(col)
            .push(
                Column::new()
                    .push(text::bold(text::simple("Other Managers xpubs:")))
                    .push(Column::with_children(other_xpubs).spacing(10))
                    .push(
                        Container::new(
                            button::white_card_button(
                                add_xpub_button,
                                button::button_content(Some(icon::plus_icon()), "Add manager"),
                            )
                            .on_press(Message::DefineManagerXpubs(
                                message::DefineManagerXpubs::AddXpub,
                            )),
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
                                add_cosigner_button,
                                button::button_content(Some(icon::plus_icon()), "Add cosigner key"),
                            )
                            .on_press(Message::DefineManagerXpubs(
                                message::DefineManagerXpubs::AddCosigner,
                            )),
                        )
                        .width(Length::Fill),
                    )
                    .spacing(10),
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

pub fn define_manager_xpubs_as_stakeholder_only<'a>(
    add_xpub_button: &'a mut Button,
    manager_xpubs: Vec<Element<'a, Message>>,
    add_cosigner_button: &'a mut Button,
    cosigners: Vec<Element<'a, Message>>,
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
            .push(text::bold(text::simple("Define managers")).size(50))
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
                            .on_press(Message::DefineManagerXpubs(
                                message::DefineManagerXpubs::AddXpub,
                            )),
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
                                add_cosigner_button,
                                button::button_content(Some(icon::plus_icon()), "Add cosigner"),
                            )
                            .on_press(Message::DefineManagerXpubs(
                                message::DefineManagerXpubs::AddCosigner,
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
