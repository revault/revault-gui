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
};

#[derive(Debug)]
pub struct ManagerHomeView {
    sidebar: ManagerSidebar,
    scroll: scrollable::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            scroll: scrollable::State::new(),
            sidebar: ManagerSidebar::new(Role::Manager, true),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        balance: &u64,
        blockheight: Option<&u64>,
    ) -> Element<'a, Message> {
        let mut vaults_col = Column::new();
        for vlt in vaults {
            vaults_col = vaults_col.push(vlt);
        }
        layout::dashboard(
            navbar(navbar_warning(warning)),
            self.sidebar.view(ManagerSidebarCurrent::Home),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(balance_view(balance))
                        .push(vaults_col)
                        .push(bitcoin_core_card(blockheight))
                        .spacing(20),
                )),
            )),
        )
        .into()
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

fn balance_view<'a, T: 'a>(balance: &u64) -> Container<'a, T> {
    Container::new(
        Row::new().push(Column::new().width(Length::Fill)).push(
            Container::new(
                Row::new()
                    .push(text::large_title(&format!(
                        "{}",
                        *balance as f64 / 100000000_f64
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
    scroll: scrollable::State,
    sidebar: ManagerSidebar,
}

impl ManagerHistoryView {
    pub fn new() -> Self {
        ManagerHistoryView {
            sidebar: ManagerSidebar::new(Role::Manager, true),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut vaults_col = Column::new();
        for vlt in vaults {
            vaults_col = vaults_col.push(vlt);
        }
        layout::dashboard(
            navbar(navbar_warning(warning)),
            self.sidebar.view(ManagerSidebarCurrent::History),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll)
                    .push(Container::new(Column::new().push(vaults_col).spacing(20))),
            )),
        )
        .into()
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
    SelectInputs,
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
    Edit {
        address_input: text_input::State,
        amount_input: text_input::State,
    },
}

impl ManagerSendOutputView {
    pub fn new_edit() -> Self {
        Self::Edit {
            address_input: text_input::State::focused(),
            amount_input: text_input::State::focused(),
        }
    }
    pub fn view(&mut self, address: &str, amount: &u64) -> Element<ManagerSendOutputMessage> {
        match self {
            Self::Edit {
                address_input,
                amount_input,
            } => {
                let address = TextInput::new(
                    address_input,
                    "address",
                    &address,
                    ManagerSendOutputMessage::AddressEdited,
                );
                let amount = TextInput::new(
                    amount_input,
                    "amount",
                    &format!("{}", amount),
                    ManagerSendOutputMessage::AmountEdited,
                );
                Container::new(Column::new().push(address).push(amount))
                    .width(Length::Units(150))
                    .into()
            }
            _ => Container::new(Column::new()).into(),
        }
    }
}
