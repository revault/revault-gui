use iced::{container, pick_list, Column, Container, Length, Row};

use revault_ui::{
    color,
    component::{button, separation, text::Text, TransparentPickListStyle},
    icon::{
        deposit_icon, history_icon, home_icon, person_check_icon, plus_icon, send_icon,
        settings_icon, vaults_icon, warning_icon,
    },
};

use crate::revault::Role;
use crate::{
    app::{context::Context, menu::Menu, message::Message},
    daemon::client::Client,
};

#[derive(Debug, Clone)]
pub struct Sidebar {
    pick_role: pick_list::State<Role>,
    deposit_menu_button: iced::button::State,
    create_vault_button: iced::button::State,
    delegate_menu_button: iced::button::State,
    emergency_menu_button: iced::button::State,
    home_menu_button: iced::button::State,
    history_menu_button: iced::button::State,
    vaults_menu_button: iced::button::State,
    network_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            deposit_menu_button: iced::button::State::new(),
            create_vault_button: iced::button::State::new(),
            delegate_menu_button: iced::button::State::new(),
            home_menu_button: iced::button::State::new(),
            history_menu_button: iced::button::State::new(),
            emergency_menu_button: iced::button::State::new(),
            vaults_menu_button: iced::button::State::new(),
            network_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
            pick_role: pick_list::State::default(),
        }
    }

    pub fn view<C: Client>(&mut self, context: &Context<C>) -> Container<Message> {
        let role = if context.role_edit {
            Container::new(
                pick_list::PickList::new(
                    &mut self.pick_role,
                    &Role::ALL[..],
                    Some(context.role),
                    Message::ChangeRole,
                )
                .text_size(20)
                .padding(10)
                .width(Length::Units(200))
                .style(TransparentPickListStyle),
            )
        } else {
            Container::new(Text::new(&context.role.to_string())).padding(10)
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
        let history_button = if context.menu == Menu::History {
            button::primary(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
            )
            .on_press(Message::Reload)
        } else {
            button::transparent(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
            )
            .on_press(Message::Menu(Menu::History))
        };
        let vaults_button = if context.menu == Menu::Vaults {
            button::primary(
                &mut self.vaults_menu_button,
                button::button_content(Some(vaults_icon()), "Vaults"),
            )
            // VaultsState supports reload
            .on_press(Message::Reload)
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
        let deposit_button = if context.menu == Menu::Deposit {
            button::primary(
                &mut self.deposit_menu_button,
                button::button_content(Some(deposit_icon()), "Deposit"),
            )
            // DepositState supports reload
            .on_press(Message::Reload)
            .width(Length::Units(200))
        } else {
            button::transparent(
                &mut self.deposit_menu_button,
                button::button_content(Some(deposit_icon()), "Deposit"),
            )
            .on_press(Message::Menu(Menu::Deposit))
            .width(Length::Units(200))
        };
        if context.role == Role::Manager {
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
                .push(deposit_button)
                .push(
                    button::transparent(
                        &mut self.create_vault_button,
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
                                .push(Text::new("Emergency").color(color::PRIMARY))
                                .spacing(10)
                                .align_items(iced::Align::Center),
                        )
                        .padding(5),
                    )
                    .on_press(Message::Menu(Menu::Emergency))
                    .width(iced::Length::Units(200)),
                ));
        }
        sidebar(
            sidebar_menu(vec![
                role.width(Length::Units(200)),
                separation().width(iced::Length::Units(200)),
                Container::new(home_button.width(Length::Units(200))),
                Container::new(history_button.width(Length::Units(200))),
                Container::new(vaults_button.width(Length::Units(200))),
                separation().width(Length::Units(200)),
                Container::new(actions.width(Length::Units(200))),
            ]),
            Container::new(settings_button),
        )
    }
}

pub fn sidebar<'a, T: 'a>(menu: Container<'a, T>, footer: Container<'a, T>) -> Container<'a, T> {
    Container::new(
        Column::new()
            .padding(10)
            .push(menu.height(Length::Fill))
            .push(footer.height(Length::Shrink)),
    )
    .style(SidebarStyle)
}

pub struct SidebarStyle;
impl container::StyleSheet for SidebarStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::FOREGROUND.into(),
            border_width: 1.0,
            border_color: color::SECONDARY,
            ..container::Style::default()
        }
    }
}

pub struct SidebarMenuStyle;
impl container::StyleSheet for SidebarMenuStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: color::FOREGROUND.into(),
            ..container::Style::default()
        }
    }
}

pub fn sidebar_menu<'a, T: 'a>(items: Vec<Container<'a, T>>) -> Container<'a, T> {
    let mut col = Column::new().padding(15).spacing(15);
    for i in items {
        col = col.push(i)
    }
    Container::new(col).style(SidebarMenuStyle)
}
