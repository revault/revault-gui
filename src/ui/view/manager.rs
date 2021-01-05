use std::rc::Rc;

use iced::{
    scrollable, Align, Column, Container, Element, HorizontalAlignment, Length, Row, Scrollable,
    Text,
};

use crate::ui::{
    color,
    component::{badge, button, card, navbar, separation, text},
    error::Error,
    image,
    message::{Message, MessageMenu},
    view::layout,
};

use crate::revaultd::model::{Vault, VaultTransactions};

#[derive(Debug, Clone)]
pub enum ManagerView {
    Home(ManagerHomeView),
    History(ManagerHistoryView),
}

#[derive(Debug, Clone)]
pub struct ManagerHomeView {
    sidebar: ManagerSidebar,
    list_vaults: VaultList,
    scroll: scrollable::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            list_vaults: VaultList::new(),
            sidebar: ManagerSidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn load(&mut self, vaults: &Vec<Rc<Vault>>, transactions: &Vec<Rc<VaultTransactions>>) {
        self.list_vaults.load(vaults, transactions);
    }

    pub fn view(
        &mut self,
        balance: u64,
        warning: Option<&Error>,
        blockheight: Option<&u64>,
    ) -> Element<Message> {
        layout::dashboard(
            navbar(navbar_warning(warning)),
            self.sidebar.view(ManagerSidebarCurrent::Home),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(balance_view(balance))
                        .push(self.list_vaults.view())
                        .push(bitcoin_core_card(blockheight))
                        .spacing(20),
                )),
            )),
        )
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

#[derive(Debug, Clone)]
struct VaultList(Vec<VaultListItem>);

impl VaultList {
    fn new() -> Self {
        VaultList(Vec::new())
    }

    fn load(&mut self, vaults: &Vec<Rc<Vault>>, transactions: &Vec<Rc<VaultTransactions>>) {
        self.0 = Vec::new();
        for vlt in vaults {
            let mut item = VaultListItem::new(vlt.clone(), None);
            for txs in transactions {
                if item.vault.outpoint() == txs.outpoint {
                    item.txs = Some(txs.clone());
                    break;
                }
            }
            self.0.push(item);
        }
    }

    fn view(&mut self) -> Container<Message> {
        if self.0.len() == 0 {
            return Container::new(Text::new("No vaults yet"));
        }
        let mut col = Column::new();
        for item in self.0.iter_mut() {
            col = col.push(item.view());
        }

        Container::new(col.spacing(10))
    }
}

#[derive(Debug, Clone)]
struct VaultListItem {
    state: iced::button::State,
    txs: Option<Rc<VaultTransactions>>,
    vault: Rc<Vault>,
}

impl VaultListItem {
    pub fn new(vault: Rc<Vault>, txs: Option<Rc<VaultTransactions>>) -> Self {
        VaultListItem {
            state: iced::button::State::new(),
            txs: txs,
            vault: vault,
        }
    }

    pub fn view<'a>(&'a mut self) -> Container<'a, Message> {
        let mut col = Column::new().push(button::transparent(
            &mut self.state,
            card::white(Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::tx_deposit())
                                .push(text::small(&self.vault.txid))
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(Text::new(format!(
                            "{}",
                            self.vault.amount as f64 / 100000000_f64
                        )))
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            )),
            Message::SelectVault(self.vault.outpoint()),
        ));
        if let Some(txs) = &self.txs {
            col = col.push(txs_card(txs));
        }
        card::rounded(Container::new(col))
    }
}

fn txs_card<'a, T: 'a>(tx: &Rc<VaultTransactions>) -> Container<'a, T> {
    let mut col = Column::new();
    col = col.push(Container::new(Text::new("HELLO")));

    Container::new(col)
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

#[derive(Debug, Clone)]
pub struct ManagerHistoryView {
    sidebar: ManagerSidebar,
    scroll: scrollable::State,
}

impl ManagerHistoryView {
    pub fn new() -> Self {
        ManagerHistoryView {
            sidebar: ManagerSidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        layout::dashboard(
            navbar(None),
            self.sidebar.view(ManagerSidebarCurrent::History),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(card::simple(text::paragraph("main"))),
            )),
        )
    }
}

#[derive(PartialEq)]
enum ManagerSidebarCurrent {
    Home,
    History,
}

#[derive(Debug, Clone)]
struct ManagerSidebar {
    home_menu_button: iced::button::State,
    history_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl ManagerSidebar {
    fn new() -> Self {
        ManagerSidebar {
            home_menu_button: iced::button::State::new(),
            history_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
        }
    }

    fn view(&mut self, current: ManagerSidebarCurrent) -> Container<Message> {
        let home_button = if current == ManagerSidebarCurrent::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(image::home_white_icon()), "Home"),
                Message::Menu(MessageMenu::Home),
            )
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(image::home_icon()), "Home"),
                Message::Menu(MessageMenu::Home),
            )
        };
        let history_button = if current == ManagerSidebarCurrent::History {
            button::primary(
                &mut self.history_menu_button,
                button::button_content(Some(image::history_white_icon()), "History"),
                Message::Menu(MessageMenu::History),
            )
        } else {
            button::transparent(
                &mut self.history_menu_button,
                button::button_content(Some(image::history_icon()), "History"),
                Message::Menu(MessageMenu::History),
            )
        };
        layout::sidebar(
            layout::sidebar_menu(vec![
                Container::new(home_button.width(iced::Length::Units(150))),
                Container::new(history_button.width(iced::Length::Units(150))),
                separation().width(iced::Length::Units(150)),
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
