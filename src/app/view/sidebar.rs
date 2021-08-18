use iced::{pick_list, Column, Container, Length, Row};

use crate::revault::Role;
use crate::{
    app::{context::Context, menu::Menu, message::Message, view::layout},
    ui::{
        color,
        component::{button, separation, text, TransparentPickListStyle},
        icon::{
            deposit_icon, home_icon, person_check_icon, plus_icon, send_icon, settings_icon,
            vaults_icon, warning_icon,
        },
    },
};

#[derive(Debug, Clone)]
pub struct Sidebar {
    pick_role: pick_list::State<Role>,
    deposit_menu_button: iced::button::State,
    delegate_menu_button: iced::button::State,
    emergency_menu_button: iced::button::State,
    home_menu_button: iced::button::State,
    vaults_menu_button: iced::button::State,
    network_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            deposit_menu_button: iced::button::State::new(),
            delegate_menu_button: iced::button::State::new(),
            home_menu_button: iced::button::State::new(),
            emergency_menu_button: iced::button::State::new(),
            vaults_menu_button: iced::button::State::new(),
            network_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
            pick_role: pick_list::State::default(),
        }
    }

    pub fn view(&mut self, context: &Context) -> Container<Message> {
        let role = if context.role_edit {
            Container::new(
                pick_list::PickList::new(
                    &mut self.pick_role,
                    &Role::ALL[..],
                    Some(context.role),
                    Message::ChangeRole,
                )
                .padding(10)
                .width(Length::Units(200))
                .style(TransparentPickListStyle),
            )
        } else {
            Container::new(text::simple(&context.role.to_string())).padding(10)
        };
        let home_button = if context.menu == Menu::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
            )
            .on_press(Message::Menu(Menu::Home))
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
            )
            .on_press(Message::Menu(Menu::Home))
        };
        let vaults_button = if context.menu == Menu::Vaults {
            button::primary(
                &mut self.vaults_menu_button,
                button::button_content(Some(vaults_icon()), "Vaults"),
            )
            .on_press(Message::Menu(Menu::Vaults))
        } else {
            button::transparent(
                &mut self.vaults_menu_button,
                button::button_content(Some(vaults_icon()), "Vaults"),
            )
            .on_press(Message::Menu(Menu::Vaults))
        };

        let settings_button = if context.menu == Menu::Settings {
            button::primary(
                &mut self.settings_menu_button,
                button::button_content(Some(settings_icon()), "Settings"),
            )
            .on_press(Message::Menu(Menu::Settings))
            .width(iced::Length::Units(200))
        } else {
            button::transparent(
                &mut self.settings_menu_button,
                button::button_content(Some(settings_icon()), "Settings"),
            )
            .on_press(Message::Menu(Menu::Settings))
            .width(iced::Length::Units(200))
        };

        let mut actions = Column::new().spacing(15);
        if context.role == Role::Manager {
            let deposit_button = if context.menu == Menu::Deposit {
                button::primary(
                    &mut self.deposit_menu_button,
                    button::button_content(Some(deposit_icon()), "Deposit"),
                )
                .on_press(Message::Menu(Menu::Deposit))
                .width(Length::Units(200))
            } else {
                button::transparent(
                    &mut self.deposit_menu_button,
                    button::button_content(Some(deposit_icon()), "Deposit"),
                )
                .on_press(Message::Menu(Menu::Deposit))
                .width(Length::Units(200))
            };

            actions = actions.push(deposit_button).push(Container::new(
                button::transparent(
                    &mut self.spend_menu_button,
                    button::button_content(Some(send_icon()), "Send"),
                )
                .on_press(Message::Menu(Menu::Send))
                .width(iced::Length::Units(200)),
            ));
        } else {
            let action_delegate = if context.menu == Menu::DelegateFunds {
                Container::new(
                    button::primary(
                        &mut self.delegate_menu_button,
                        button::button_content(Some(person_check_icon()), "Delegate funds"),
                    )
                    .on_press(Message::Menu(Menu::DelegateFunds))
                    .width(iced::Length::Units(200)),
                )
            } else {
                Container::new(
                    button::transparent(
                        &mut self.delegate_menu_button,
                        button::button_content(Some(person_check_icon()), "Delegate funds"),
                    )
                    .on_press(Message::Menu(Menu::DelegateFunds))
                    .width(iced::Length::Units(200)),
                )
            };
            actions = actions
                .push(
                    button::transparent(
                        &mut self.deposit_menu_button,
                        button::button_content(Some(plus_icon()), "Create vault"),
                    )
                    .on_press(Message::Menu(Menu::CreateVaults))
                    .width(iced::Length::Units(200)),
                )
                .push(action_delegate)
                .push(Container::new(
                    button::transparent(
                        &mut self.emergency_menu_button,
                        Container::new(
                            Row::new()
                                .push(warning_icon().color(color::PRIMARY))
                                .push(text::simple("Emergency").color(color::PRIMARY))
                                .spacing(10)
                                .align_items(iced::Align::Center),
                        )
                        .padding(5),
                    )
                    .on_press(Message::Menu(Menu::Emergency))
                    .width(iced::Length::Units(200)),
                ));
        }
        layout::sidebar(
            layout::sidebar_menu(vec![
                role.width(Length::Units(200)),
                separation().width(iced::Length::Units(200)),
                Container::new(home_button.width(Length::Units(200))),
                Container::new(vaults_button.width(Length::Units(200))),
                separation().width(Length::Units(200)),
                Container::new(actions.width(Length::Units(200))),
            ]),
            Container::new(settings_button),
        )
    }
}
