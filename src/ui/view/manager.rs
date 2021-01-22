use iced::{
    pick_list, scrollable, text_input, Checkbox, Column, Container, Element, HorizontalAlignment,
    Length, Row, Scrollable, TextInput,
};

use crate::ui::{
    color,
    component::{badge, button, card, navbar, separation, text, TransparentPickListStyle},
    error::Error,
    icon::{dot_icon, history_icon, home_icon, send_icon, settings_icon},
    message::{InputMessage, Menu, Message, RecipientMessage, Role},
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
    SelectInputs(ManagerSelectInputsView),
    SelectFee(ManagerSelectFeeView),
}

impl ManagerSendView {
    pub fn new() -> Self {
        Self::SelectOutputs(ManagerSelectOutputsView::new())
    }

    pub fn next(&self) -> ManagerSendView {
        match self {
            Self::SelectOutputs(_) => Self::SelectInputs(ManagerSelectInputsView::new()),
            Self::SelectInputs(_) => Self::SelectFee(ManagerSelectFeeView::new()),
            _ => Self::new(),
        }
    }

    pub fn previous(&self) -> ManagerSendView {
        match self {
            Self::SelectInputs(_) => Self::SelectOutputs(ManagerSelectOutputsView::new()),
            Self::SelectFee(_) => Self::SelectInputs(ManagerSelectInputsView::new()),
            _ => Self::new(),
        }
    }
}

#[derive(Debug)]
pub struct ManagerSelectOutputsView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    new_output_button: iced::button::State,
}

impl ManagerSelectOutputsView {
    pub fn new() -> Self {
        ManagerSelectOutputsView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            new_output_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        selected_outputs: Vec<Element<'a, Message>>,
        valid: bool,
    ) -> Element<'a, Message> {
        let mut col_outputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(iced::Align::Center);
        for (i, element) in selected_outputs.into_iter().enumerate() {
            if i > 0 {
                col_outputs = col_outputs.push(separation().width(Length::Fill));
            }
            col_outputs = col_outputs.push(element);
        }
        let element: Element<_> = col_outputs.max_width(500).into();

        let mut footer = Row::new().spacing(20).push(button::cancel(
            &mut self.new_output_button,
            Container::new(text::simple("Add recipient")).padding(10),
            Message::AddRecipient,
        ));

        if valid {
            footer = footer.push(Container::new(button::primary(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::Next,
            )));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::None,
            )));
        }
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
                    .push(
                        Container::new(element)
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct ManagerSendOutputView {
    address_input: text_input::State,
    amount_input: text_input::State,
    delete_button: iced::button::State,
}

impl ManagerSendOutputView {
    pub fn new() -> Self {
        Self {
            address_input: text_input::State::focused(),
            amount_input: text_input::State::new(),
            delete_button: iced::button::State::new(),
        }
    }
    pub fn view(
        &mut self,
        address: &str,
        amount: &str,
        warning_address: &bool,
        warning_amount: &bool,
    ) -> Element<RecipientMessage> {
        let address = TextInput::new(
            &mut self.address_input,
            "Address",
            &address,
            RecipientMessage::AddressEdited,
        )
        .padding(10);
        let mut col = Column::new()
            .push(
                Container::new(button::transparent(
                    &mut self.delete_button,
                    Container::new(text::simple("X Remove")).padding(10),
                    RecipientMessage::Delete,
                ))
                .width(Length::Fill)
                .align_x(iced::Align::End),
            )
            .push(text::bold("Enter address:"))
            .push(address);

        if *warning_address {
            col = col.push(card::alert_warning(Container::new(text::simple(
                "Please enter a valid bitcoin address",
            ))))
        }
        col = col.push(text::bold("Enter amount:")).push(
            TextInput::new(
                &mut self.amount_input,
                "0.0",
                &format!("{}", amount),
                RecipientMessage::AmountEdited,
            )
            .padding(10),
        );

        if *warning_amount {
            col = col.push(card::alert_warning(Container::new(text::simple(
                "Please enter a valid amount",
            ))))
        }
        Container::new(col.spacing(10)).into()
    }
}

#[derive(Debug)]
pub struct ManagerSelectInputsView {
    scroll: scrollable::State,
    back_button: iced::button::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    new_output_button: iced::button::State,
}

impl ManagerSelectInputsView {
    pub fn new() -> Self {
        ManagerSelectInputsView {
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            new_output_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        selected_inputs: Vec<Element<'a, Message>>,
        valid: bool,
    ) -> Element<'a, Message> {
        let mut col_inputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(iced::Align::Center);
        for (i, element) in selected_inputs.into_iter().enumerate() {
            if i > 0 {
                col_inputs = col_inputs.push(separation().width(Length::Fill));
            }
            col_inputs = col_inputs.push(element);
        }
        let element: Element<_> = col_inputs.max_width(500).into();

        let mut footer = Column::new();
        if valid {
            footer = footer.push(Container::new(button::primary(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::Next,
            )));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::None,
            )));
        }
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(button::transparent(
                                        &mut self.back_button,
                                        Container::new(text::simple("Go Back")).padding(10),
                                        Message::Previous,
                                    ))
                                    .width(Length::Fill),
                            )
                            .push(
                                Container::new(button::cancel(
                                    &mut self.cancel_button,
                                    Container::new(text::simple("X Close")).padding(10),
                                    Message::Menu(Menu::Home),
                                ))
                                .width(Length::Shrink),
                            ),
                    )
                    .push(
                        Container::new(element)
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub fn manager_send_input_view<'a>(
    outpoint: &str,
    amount: &u64,
    selected: bool,
) -> Element<'a, InputMessage> {
    let checkbox =
        Checkbox::new(selected, &format!("{}", outpoint), InputMessage::Selected).text_size(15);
    let row = Row::new()
        .push(checkbox)
        .push(text::bold(&format!("{}", *amount as f64 / 100000000_f64)))
        .spacing(20);
    Container::new(row).width(Length::Fill).into()
}

#[derive(Debug)]
pub struct ManagerSelectFeeView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerSelectFeeView {
    pub fn new() -> Self {
        ManagerSelectFeeView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(&'a mut self, valid: bool) -> Element<'a, Message> {
        let mut footer = Row::new().spacing(20);
        if valid {
            footer = footer.push(Container::new(button::primary(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::Next,
            )));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
                Message::None,
            )));
        }
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(button::transparent(
                                        &mut self.back_button,
                                        Container::new(text::simple("Go Back")).padding(10),
                                        Message::Previous,
                                    ))
                                    .width(Length::Fill),
                            )
                            .push(
                                Container::new(button::cancel(
                                    &mut self.cancel_button,
                                    Container::new(text::simple("X Close")).padding(10),
                                    Message::Menu(Menu::Home),
                                ))
                                .width(Length::Shrink),
                            ),
                    )
                    .push(
                        Container::new(text::simple("Select fee"))
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
