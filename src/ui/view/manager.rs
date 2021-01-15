use std::rc::Rc;

use iced::{
    pick_list, scrollable, text_input, Column, Container, Element, HorizontalAlignment, Length,
    Row, Scrollable, TextInput,
};

use crate::ui::{
    color,
    component::{badge, button, card, navbar, separation, text, TransparentPickListStyle},
    error::Error,
    icon::{dot_icon, history_icon, home_icon, send_icon, settings_icon},
    message::{ManagerSendOutputMessage, Menu, Message, Role},
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
        vaults: Vec<Rc<(Vault, VaultTransactions)>>,
        selected_vault: Option<Rc<(Vault, VaultTransactions)>>,
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
        return Some(card::alert_warning(Container::new(text::simple(&format!(
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
                Row::new()
                    .push(text::large_title(&format!(
                        "{}",
                        balance as f64 / 100000000_f64
                    )))
                    .push(text::simple(" BTC"))
                    .align_items(iced::Align::Center),
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
                .push(Container::new(text::bold("Bitcoin Core")).width(Length::Fill))
                .push(
                    Container::new(
                        Row::new()
                            .push(dot_icon().size(5).color(color::SUCCESS))
                            .push(text::small("Running").color(color::SUCCESS))
                            .align_items(iced::Align::Center),
                    )
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
                        .push(text::bold("Block Height"))
                        .push(text::simple(&format!("{}", b))),
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
        vaults: Vec<Rc<(Vault, VaultTransactions)>>,
        selected_vault: Option<Rc<(Vault, VaultTransactions)>>,
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
            Container::new(text::simple(&format!("{}", self.role)))
        };
        let home_button = if current == ManagerSidebarCurrent::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        };
        let history_button = if current == ManagerSidebarCurrent::History {
            button::primary(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
                Message::Menu(Menu::History),
            )
        } else {
            button::transparent(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
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
                        button::button_content(Some(send_icon()), "Spend"),
                        Message::Menu(Menu::Send),
                    )
                    .width(iced::Length::Units(150)),
                ),
            ]),
            Container::new(
                button::transparent(
                    &mut self.settings_menu_button,
                    button::button_content(Some(settings_icon()), "Settings"),
                    Message::Install,
                )
                .width(iced::Length::Units(150)),
            ),
        )
    }
}

#[derive(Debug)]
pub enum ManagerSendView {
    SelectOutputs(ManagerSelectOutputsView),
    SelectInputs(ManagerSelectInputsView),
}

impl ManagerSendView {
    pub fn new() -> Self {
        Self::SelectOutputs(ManagerSelectOutputsView::new())
    }
}

#[derive(Debug)]
pub struct ManagerSelectOutputsView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
}

impl ManagerSelectOutputsView {
    pub fn new() -> Self {
        ManagerSelectOutputsView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        selected_outputs: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut col_outputs = Column::new();
        for element in selected_outputs {
            col_outputs = col_outputs.push(element);
        }
        let element: Element<_> = col_outputs.into();
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(button::cancel(
                                &mut self.cancel_button,
                                Container::new(text::simple("X Close")).padding(10),
                                Message::Menu(Menu::Home),
                            ))
                            .width(Length::Shrink),
                        ),
                    )
                    .push(element)
                    .push(Container::new(button::primary(
                        &mut self.next_button,
                        Container::new(text::simple("Continue")),
                        Message::Next,
                    ))),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub enum ManagerSendOutputView {
    Display,
    Edit { address_input: text_input::State },
}

impl ManagerSendOutputView {
    pub fn new_edit() -> Self {
        Self::Edit {
            address_input: text_input::State::focused(),
        }
    }
    pub fn view(&mut self, address: &str) -> Element<ManagerSendOutputMessage> {
        match self {
            Self::Edit { address_input } => {
                let input = TextInput::new(
                    address_input,
                    "address",
                    &address,
                    ManagerSendOutputMessage::AddressEdited,
                );
                Container::new(input).width(Length::Fill).into()
            }
            _ => Container::new(Column::new()).into(),
        }
    }
}

#[derive(Debug)]
pub struct ManagerSelectInputsView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    list_vaults: Vec<Rc<(Vault, VaultTransactions)>>,
    warning: Option<Error>,
}

impl ManagerSelectInputsView {
    pub fn new() -> Self {
        ManagerSelectInputsView {
            list_vaults: Vec::new(),
            cancel_button: iced::button::State::new(),
            warning: None,
            scroll: scrollable::State::new(),
        }
    }
    pub fn load(&mut self, vaults: Vec<Rc<(Vault, VaultTransactions)>>, warning: Option<Error>) {
        self.list_vaults = vaults;
        self.warning = warning;
    }

    pub fn view(&mut self) -> Element<Message> {
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new().push(
                    Row::new().push(Column::new().width(Length::Fill)).push(
                        Container::new(button::cancel(
                            &mut self.cancel_button,
                            Container::new(text::simple("X Close")).padding(10),
                            Message::Menu(Menu::Home),
                        ))
                        .width(Length::Shrink),
                    ),
                ),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
