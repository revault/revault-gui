use std::rc::Rc;

use iced::{
    pick_list, scrollable, Column, Container, Element, HorizontalAlignment, Length, Row,
    Scrollable, Text,
};

use crate::ui::{
    color,
    component::{badge, button, card, navbar, separation, text, TransparentPickListStyle},
    error::Error,
    image,
    message::{Menu, Message, Role},
    view::layout,
    view::vault::{VaultList, VaultModal},
};

use crate::revaultd::model::{Vault, VaultTransactions};

#[derive(Debug)]
pub struct ManagerHomeView {
    balance: u64,
    blockheight: Option<u64>,
    list_vaults: VaultList,
    modal: VaultModal,
    sidebar: ManagerSidebar,
    warning: Option<Error>,

    scroll: scrollable::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            balance: 0,
            blockheight: None,
            list_vaults: VaultList::new(),
            modal: VaultModal::new(),
            scroll: scrollable::State::new(),
            sidebar: ManagerSidebar::new(Role::Manager, true),
            warning: None,
        }
    }

    pub fn load(
        &mut self,
        vaults: Vec<Rc<Vault>>,
        selected_vault: Option<(Rc<Vault>, VaultTransactions)>,
        balance: u64,
        blockheight: Option<u64>,
        warning: Option<Error>,
    ) {
        self.modal.load(selected_vault);
        self.list_vaults.load(vaults);
        self.warning = warning;
        self.balance = balance;
        self.blockheight = blockheight;
    }

    pub fn view(&mut self) -> Element<Message> {
        let background = layout::dashboard(
            navbar(navbar_warning(self.warning.as_ref())),
            self.sidebar.view(ManagerSidebarCurrent::Home),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(balance_view(self.balance))
                        .push(self.list_vaults.view())
                        .push(bitcoin_core_card(self.blockheight.as_ref()))
                        .spacing(20),
                )),
            )),
        );

        self.modal.view(background).into()
    }
}

fn navbar_warning<'a, T: 'a>(warning: Option<&Error>) -> Option<Container<'a, T>> {
    if let Some(e) = warning {
        return Some(card::alert_warning(Container::new(Text::new(format!(
            "{}",
            e
        )))));
    }
    None
}

fn balance_view<'a, T: 'a>(balance: u64) -> Container<'a, T> {
    Container::new(
        Row::new().push(Column::new().width(Length::Fill)).push(
            Container::new(
                text::large_title(&format!("{}", balance as f64 / 100000000_f64))
                    .horizontal_alignment(HorizontalAlignment::Right),
            )
            .width(Length::Shrink),
        ),
    )
    .width(Length::Fill)
}

fn bitcoin_core_card<'a, T: 'a>(blockheight: Option<&u64>) -> Container<'a, T> {
    let mut col = Column::new()
        .push(
            Row::new()
                .push(Container::new(Text::new("Bitcoin Core")).width(Length::Fill))
                .push(
                    Container::new(text::small("* running").color(color::SUCCESS))
                        .width(Length::Shrink),
                ),
        )
        .spacing(10);
    if let Some(b) = blockheight {
        col = col.push(
            Row::new()
                .push(badge::block())
                .push(
                    Column::new()
                        .push(text::small("Block Height"))
                        .push(Text::new(&format!("{}", b))),
                )
                .spacing(10),
        );
    }
    card::simple(Container::new(col))
}

#[derive(Debug)]
pub struct ManagerHistoryView {
    list_vaults: VaultList,
    modal: VaultModal,
    scroll: scrollable::State,
    sidebar: ManagerSidebar,
    warning: Option<Error>,
}

impl ManagerHistoryView {
    pub fn new() -> Self {
        ManagerHistoryView {
            modal: VaultModal::new(),
            list_vaults: VaultList::new(),
            sidebar: ManagerSidebar::new(Role::Manager, true),
            scroll: scrollable::State::new(),
            warning: None,
        }
    }

    pub fn load(
        &mut self,
        vaults: Vec<Rc<Vault>>,
        selected_vault: Option<(Rc<Vault>, VaultTransactions)>,
        warning: Option<Error>,
    ) {
        self.modal.load(selected_vault);
        self.list_vaults.load(vaults);
        self.warning = warning;
    }

    pub fn view(&mut self) -> Element<Message> {
        let background = layout::dashboard(
            navbar(navbar_warning(self.warning.as_ref())),
            self.sidebar.view(ManagerSidebarCurrent::History),
            layout::main_section(Container::new(Scrollable::new(&mut self.scroll).push(
                Container::new(Column::new().push(self.list_vaults.view()).spacing(20)),
            ))),
        );

        self.modal.view(background).into()
    }
}

#[derive(PartialEq)]
enum ManagerSidebarCurrent {
    Home,
    History,
}

#[derive(Debug, Clone)]
struct ManagerSidebar {
    role: Role,
    edit: bool,
    pick_role: pick_list::State<Role>,
    home_menu_button: iced::button::State,
    history_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl ManagerSidebar {
    fn new(role: Role, edit: bool) -> Self {
        ManagerSidebar {
            role,
            edit,
            home_menu_button: iced::button::State::new(),
            history_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
            pick_role: pick_list::State::default(),
        }
    }

    fn view(&mut self, current: ManagerSidebarCurrent) -> Container<Message> {
        let role = if self.edit {
            Container::new(
                pick_list::PickList::new(
                    &mut self.pick_role,
                    &Role::ALL[..],
                    Some(self.role),
                    Message::ChangeRole,
                )
                .padding(10)
                .width(Length::Units(200))
                .style(TransparentPickListStyle),
            )
        } else {
            Container::new(Text::new(format!("{}", self.role)))
        };
        let home_button = if current == ManagerSidebarCurrent::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(image::home_white_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(image::home_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        };
        let history_button = if current == ManagerSidebarCurrent::History {
            button::primary(
                &mut self.history_menu_button,
                button::button_content(Some(image::history_white_icon()), "History"),
                Message::Menu(Menu::History),
            )
        } else {
            button::transparent(
                &mut self.history_menu_button,
                button::button_content(Some(image::history_icon()), "History"),
                Message::Menu(Menu::History),
            )
        };
        layout::sidebar(
            layout::sidebar_menu(vec![
                role.width(Length::Units(200)),
                separation().width(iced::Length::Units(200)),
                Container::new(home_button.width(Length::Units(200))),
                Container::new(history_button.width(Length::Units(200))),
                separation().width(Length::Units(200)),
                Container::new(
                    button::transparent(
                        &mut self.spend_menu_button,
                        button::button_content(Some(image::send_icon()), "Spend"),
                        Message::Install,
                    )
                    .width(iced::Length::Units(150)),
                ),
            ]),
            Container::new(
                button::transparent(
                    &mut self.settings_menu_button,
                    button::button_content(Some(image::settings_icon()), "Settings"),
                    Message::Install,
                )
                .width(iced::Length::Units(150)),
            ),
        )
    }
}
