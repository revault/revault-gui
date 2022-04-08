use std::convert::From;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use iced::{Command, Element};

use bitcoin::hashes::hex::{FromHex, ToHex};
use revault_ui::component::form;
use revaultd::{config::WatchtowerConfig, revault_net::noise::PublicKey as NoisePubkey};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, SettingsMessage},
        state::cmd::get_server_status,
        state::State,
        view::settings::*,
    },
    daemon::model::ServersStatuses,
    revault::Role,
};

trait Setting: std::fmt::Debug {
    fn edited(&mut self, success: bool);
    fn update(&mut self, ctx: &Context, message: SettingsMessage) -> Command<Message>;
    fn view(
        &mut self,
        ctx: &Context,
        statuses: &Option<ServersStatuses>,
        can_edit: bool,
    ) -> Element<SettingsMessage>;
}

#[derive(Debug)]
pub struct SettingsState {
    warning: Option<Error>,
    view: SettingsView,
    server_statuses: Option<ServersStatuses>,
    config_updated: bool,

    settings: Vec<Box<dyn Setting>>,
    current: Option<usize>,
}

impl SettingsState {
    pub fn new(ctx: &Context) -> Self {
        let mut settings = vec![
            BitcoindSettings::default().into(),
            CoordinatorSettings::default().into(),
        ];

        if ctx.role == Role::Stakeholder {
            if let Some(cfg) = &ctx.config.daemon.stakeholder_config {
                for i in 0..cfg.watchtowers.len() {
                    settings.push(WatchtowerSettings::new(i).into());
                }
            }
        } else if let Some(cfg) = &ctx.config.daemon.manager_config {
            for i in 0..cfg.cosigners.len() {
                settings.push(CosignerSettings::new(i).into());
            }
        }
        SettingsState {
            view: SettingsView::default(),
            warning: None,
            server_statuses: None,
            config_updated: false,
            settings,
            current: None,
        }
    }
}

impl State for SettingsState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::ServerStatus(s) => {
                match s {
                    Ok(statuses) => self.server_statuses = Some(statuses),
                    Err(e) => self.warning = Error::from(e).into(),
                };
            }
            Message::DaemonConfigLoaded(res) => match res {
                Ok(()) => {
                    self.config_updated = true;
                    if let Some(current) = self.current {
                        if let Some(setting) = self.settings.get_mut(current) {
                            setting.edited(true)
                        }
                    }
                    self.current = None;
                    return Command::perform(
                        get_server_status(ctx.revaultd.clone()),
                        Message::ServerStatus,
                    );
                }
                Err(e) => {
                    self.config_updated = false;
                    self.warning = Some(e);
                    if let Some(current) = self.current {
                        if let Some(setting) = self.settings.get_mut(current) {
                            setting.edited(false);
                        }
                    }
                }
            },
            Message::Settings(i, SettingsMessage::Remove) => {
                if Some(i) == self.current {
                    self.current = None;
                }
                self.settings.remove(i);
            }
            Message::Settings(i, msg) => {
                if let Some(setting) = self.settings.get_mut(i) {
                    match msg {
                        SettingsMessage::Edit => self.current = Some(i),
                        SettingsMessage::CancelEdit => self.current = None,
                        _ => {}
                    };
                    return setting.update(ctx, msg);
                }
            }
            Message::AddWatchtower => {
                if ctx.role == Role::Stakeholder {
                    self.settings.push(
                        WatchtowerSettings::Edit {
                            index: self.settings.len() - 2,
                            processing: false,
                            key: form::Value::default(),
                            host: form::Value::default(),
                            view: WatchtowerSettingsEditView::default(),
                        }
                        .into(),
                    );
                    self.current = Some(self.settings.len() - 1);
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        let server_statuses = &self.server_statuses;
        let can_edit = self.current.is_none() && !ctx.revaultd.is_external();
        self.view.view(
            ctx,
            self.warning.as_ref(),
            can_edit,
            self.settings
                .iter_mut()
                .enumerate()
                .map(|(i, setting)| {
                    setting
                        .view(ctx, server_statuses, can_edit)
                        .map(move |msg| Message::Settings(i, msg))
                })
                .collect(),
        )
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_server_status(ctx.revaultd.clone()),
            Message::ServerStatus,
        )])
    }
}

impl From<SettingsState> for Box<dyn State> {
    fn from(s: SettingsState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub enum BitcoindSettings {
    Display(BitcoindSettingsView),
    Edit {
        processing: bool,
        cookie_path: form::Value<String>,
        addr: form::Value<String>,
        view: BitcoindSettingsEditView,
    },
}

impl Default for BitcoindSettings {
    fn default() -> Self {
        Self::Display(BitcoindSettingsView::default())
    }
}

impl From<BitcoindSettings> for Box<dyn Setting> {
    fn from(s: BitcoindSettings) -> Box<dyn Setting> {
        Box::new(s)
    }
}

impl Setting for BitcoindSettings {
    fn edited(&mut self, success: bool) {
        if success {
            *self = Self::default();
        } else if let Self::Edit { processing, .. } = self {
            *processing = false;
        }
    }

    fn update(&mut self, ctx: &Context, message: SettingsMessage) -> Command<Message> {
        if matches!(message, SettingsMessage::Edit) {
            *self = Self::Edit {
                processing: false,
                cookie_path: form::Value {
                    valid: true,
                    value: ctx
                        .config
                        .daemon
                        .bitcoind_config
                        .cookie_path
                        .to_str()
                        .unwrap()
                        .to_string(),
                },
                addr: form::Value {
                    valid: true,
                    value: ctx.config.daemon.bitcoind_config.addr.to_string(),
                },
                view: BitcoindSettingsEditView::default(),
            };
        }
        if let Self::Edit {
            addr,
            cookie_path,
            processing,
            ..
        } = self
        {
            match message {
                SettingsMessage::Edit | SettingsMessage::Remove => {}
                SettingsMessage::CancelEdit => {
                    if !*processing {
                        *self = Self::default();
                    }
                }
                SettingsMessage::FieldEdited(field, value) => {
                    if !*processing {
                        match field {
                            "socket_address" => addr.value = value,
                            "cookie_file_path" => cookie_path.value = value,
                            _ => {}
                        }
                    }
                }
                SettingsMessage::ConfirmEdit => {
                    let new_addr = SocketAddr::from_str(&addr.value);
                    addr.valid = new_addr.is_ok();
                    let new_path = PathBuf::from_str(&cookie_path.value);
                    cookie_path.valid = new_path.is_ok();

                    if addr.valid & cookie_path.valid {
                        let mut daemon_config = ctx.config.daemon.clone();
                        daemon_config.bitcoind_config.cookie_path = new_path.unwrap();
                        daemon_config.bitcoind_config.addr = new_addr.unwrap();
                        *processing = true;
                        return Command::perform(async move { daemon_config }, |cfg| {
                            Message::LoadDaemonConfig(cfg)
                        });
                    }
                }
            };
        }
        Command::none()
    }

    fn view(
        &mut self,
        ctx: &Context,
        _statuses: &Option<ServersStatuses>,
        can_edit: bool,
    ) -> Element<SettingsMessage> {
        match self {
            Self::Display(v) => v.view(
                &ctx.config.daemon.bitcoind_config,
                ctx.blockheight,
                Some(ctx.blockheight != 0),
                can_edit,
            ),
            Self::Edit {
                view,
                addr,
                cookie_path,
                processing,
            } => view.view(
                &ctx.config.daemon.bitcoind_config,
                ctx.blockheight,
                &addr,
                &cookie_path,
                *processing,
            ),
        }
    }
}

#[derive(Debug)]
pub enum CoordinatorSettings {
    Display(CoordinatorSettingsView),
    Edit {
        processing: bool,
        host: form::Value<String>,
        key: form::Value<String>,
        view: CoordinatorSettingsEditView,
    },
}

impl Default for CoordinatorSettings {
    fn default() -> Self {
        Self::Display(CoordinatorSettingsView::default())
    }
}

impl From<CoordinatorSettings> for Box<dyn Setting> {
    fn from(s: CoordinatorSettings) -> Box<dyn Setting> {
        Box::new(s)
    }
}

impl Setting for CoordinatorSettings {
    fn edited(&mut self, success: bool) {
        if success {
            *self = Self::default();
        } else if let Self::Edit { processing, .. } = self {
            *processing = false;
        }
    }

    fn update(&mut self, ctx: &Context, message: SettingsMessage) -> Command<Message> {
        if matches!(message, SettingsMessage::Edit) {
            *self = Self::Edit {
                processing: false,
                key: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_noise_key.as_ref().to_hex(),
                },
                host: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_host.to_string(),
                },
                view: CoordinatorSettingsEditView::default(),
            };
        }
        if let Self::Edit {
            host,
            key,
            processing,
            ..
        } = self
        {
            match message {
                SettingsMessage::Edit | SettingsMessage::Remove => {}
                SettingsMessage::CancelEdit => {
                    if !*processing {
                        *self = Self::default();
                    }
                }
                SettingsMessage::FieldEdited(field, value) => {
                    if !*processing {
                        match field {
                            "host" => host.value = value,
                            "key" => key.value = value,
                            _ => {}
                        }
                    }
                }
                SettingsMessage::ConfirmEdit => {
                    let new_host = SocketAddr::from_str(&host.value);
                    host.valid = new_host.is_ok();
                    let new_key: Option<NoisePubkey> =
                        FromHex::from_hex(&key.value).map(NoisePubkey).ok();
                    key.valid = new_key.is_some();

                    if host.valid & key.valid {
                        let mut daemon_config = ctx.config.daemon.clone();
                        daemon_config.coordinator_host = new_host.unwrap();
                        daemon_config.coordinator_noise_key = new_key.unwrap();
                        *processing = true;
                        return Command::perform(async move { daemon_config }, |cfg| {
                            Message::LoadDaemonConfig(cfg)
                        });
                    }
                }
            };
        }
        Command::none()
    }

    fn view(
        &mut self,
        ctx: &Context,
        statuses: &Option<ServersStatuses>,
        can_edit: bool,
    ) -> Element<SettingsMessage> {
        match self {
            Self::Display(v) => v.view(
                &ctx.config.daemon.coordinator_host.to_string(),
                &ctx.config.daemon.coordinator_noise_key.as_ref().to_hex(),
                statuses.as_ref().map(|s| s.coordinator.reachable),
                can_edit,
            ),
            Self::Edit {
                view,
                host,
                key,
                processing,
            } => view.view(&host, &key, *processing),
        }
    }
}

#[derive(Debug)]
pub enum WatchtowerSettings {
    Display(usize, WatchtowerSettingsView),
    Edit {
        index: usize,
        processing: bool,
        host: form::Value<String>,
        key: form::Value<String>,
        view: WatchtowerSettingsEditView,
    },
}

impl WatchtowerSettings {
    fn new(index: usize) -> Self {
        Self::Display(index, WatchtowerSettingsView::default())
    }

    fn index(&self) -> usize {
        match self {
            Self::Display(i, _) => *i,
            Self::Edit { index, .. } => *index,
        }
    }
}

impl Setting for WatchtowerSettings {
    fn edited(&mut self, success: bool) {
        if success {
            *self = Self::new(self.index());
        } else if let Self::Edit { processing, .. } = self {
            *processing = false;
        }
    }

    fn update(&mut self, ctx: &Context, message: SettingsMessage) -> Command<Message> {
        if matches!(message, SettingsMessage::Edit) {
            *self = Self::Edit {
                index: self.index(),
                processing: false,
                key: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_noise_key.as_ref().to_hex(),
                },
                host: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_host.to_string(),
                },
                view: WatchtowerSettingsEditView::default(),
            };
        }
        if let Self::Edit {
            host,
            key,
            processing,
            index,
            ..
        } = self
        {
            match message {
                SettingsMessage::Edit | SettingsMessage::Remove => {}
                SettingsMessage::CancelEdit => {
                    if !*processing {
                        *self = Self::new(self.index());
                    }
                }
                SettingsMessage::FieldEdited(field, value) => {
                    if !*processing {
                        match field {
                            "host" => host.value = value,
                            "key" => key.value = value,
                            _ => {}
                        }
                    }
                }
                SettingsMessage::ConfirmEdit => {
                    let new_host = SocketAddr::from_str(&host.value);
                    host.valid = new_host.is_ok();
                    let new_key: Option<NoisePubkey> =
                        FromHex::from_hex(&key.value).map(NoisePubkey).ok();
                    key.valid = new_key.is_some();

                    if host.valid & key.valid {
                        let mut stakeholder_config =
                            ctx.config.daemon.stakeholder_config.clone().unwrap();
                        if let Some(wt) = stakeholder_config.watchtowers.get_mut(*index) {
                            wt.host = new_host.unwrap();
                            wt.noise_key = new_key.unwrap();
                        } else {
                            stakeholder_config.watchtowers.push(WatchtowerConfig {
                                host: new_host.unwrap(),
                                noise_key: new_key.unwrap(),
                            })
                        }
                        let mut daemon_config = ctx.config.daemon.clone();
                        daemon_config.stakeholder_config = Some(stakeholder_config);
                        *processing = true;
                        return Command::perform(async move { daemon_config }, |cfg| {
                            Message::LoadDaemonConfig(cfg)
                        });
                    }
                }
            };
        }
        Command::none()
    }

    fn view(
        &mut self,
        ctx: &Context,
        statuses: &Option<ServersStatuses>,
        can_edit: bool,
    ) -> Element<SettingsMessage> {
        match self {
            Self::Display(i, v) => {
                let wt = ctx
                    .config
                    .daemon
                    .stakeholder_config
                    .as_ref()
                    .unwrap()
                    .watchtowers
                    .get(*i)
                    .unwrap();
                v.view(
                    &wt.host.to_string(),
                    &wt.noise_key.as_ref().to_hex(),
                    statuses
                        .as_ref()
                        .map(|s| s.watchtowers.get(*i).map(|r| r.reachable).unwrap_or(false)),
                    can_edit,
                )
            }
            Self::Edit {
                view,
                host,
                key,
                processing,
                index,
            } => {
                let not_saved = ctx
                    .config
                    .daemon
                    .stakeholder_config
                    .as_ref()
                    .unwrap()
                    .watchtowers
                    .get(*index)
                    .is_none();
                view.view(not_saved, &host, &key, *processing)
            }
        }
    }
}

impl From<WatchtowerSettings> for Box<dyn Setting> {
    fn from(s: WatchtowerSettings) -> Box<dyn Setting> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub enum CosignerSettings {
    Display(usize, CosignerSettingsView),
    Edit {
        index: usize,
        processing: bool,
        host: form::Value<String>,
        key: form::Value<String>,
        view: CosignerSettingsEditView,
    },
}

impl CosignerSettings {
    fn new(index: usize) -> Self {
        Self::Display(index, CosignerSettingsView::default())
    }

    fn index(&self) -> usize {
        match self {
            Self::Display(i, _) => *i,
            Self::Edit { index, .. } => *index,
        }
    }
}

impl Setting for CosignerSettings {
    fn edited(&mut self, success: bool) {
        if success {
            *self = Self::new(self.index());
        } else if let Self::Edit { processing, .. } = self {
            *processing = false;
        }
    }

    fn update(&mut self, ctx: &Context, message: SettingsMessage) -> Command<Message> {
        if matches!(message, SettingsMessage::Edit) {
            *self = Self::Edit {
                index: self.index(),
                processing: false,
                key: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_noise_key.as_ref().to_hex(),
                },
                host: form::Value {
                    valid: true,
                    value: ctx.config.daemon.coordinator_host.to_string(),
                },
                view: CosignerSettingsEditView::default(),
            };
        }
        if let Self::Edit {
            host,
            key,
            processing,
            index,
            ..
        } = self
        {
            match message {
                SettingsMessage::Edit | SettingsMessage::Remove => {}
                SettingsMessage::CancelEdit => {
                    if !*processing {
                        *self = Self::new(self.index());
                    }
                }
                SettingsMessage::FieldEdited(field, value) => {
                    if !*processing {
                        match field {
                            "host" => host.value = value,
                            "key" => key.value = value,
                            _ => {}
                        }
                    }
                }
                SettingsMessage::ConfirmEdit => {
                    let new_host = SocketAddr::from_str(&host.value);
                    host.valid = new_host.is_ok();
                    let new_key: Option<NoisePubkey> =
                        FromHex::from_hex(&key.value).map(NoisePubkey).ok();
                    key.valid = new_key.is_some();

                    if host.valid & key.valid {
                        let mut manager_config = ctx.config.daemon.manager_config.clone().unwrap();
                        if let Some(cs) = manager_config.cosigners.get_mut(*index) {
                            cs.host = new_host.unwrap();
                            cs.noise_key = new_key.unwrap();
                        }
                        let mut daemon_config = ctx.config.daemon.clone();
                        daemon_config.manager_config = Some(manager_config);
                        *processing = true;
                        return Command::perform(async move { daemon_config }, |cfg| {
                            Message::LoadDaemonConfig(cfg)
                        });
                    }
                }
            };
        }
        Command::none()
    }

    fn view(
        &mut self,
        ctx: &Context,
        statuses: &Option<ServersStatuses>,
        can_edit: bool,
    ) -> Element<SettingsMessage> {
        match self {
            Self::Display(i, v) => {
                let cs = ctx
                    .config
                    .daemon
                    .manager_config
                    .as_ref()
                    .unwrap()
                    .cosigners
                    .get(*i)
                    .unwrap();
                v.view(
                    &cs.host.to_string(),
                    &cs.noise_key.as_ref().to_hex(),
                    statuses
                        .as_ref()
                        .map(|s| s.cosigners.get(*i).map(|r| r.reachable).unwrap_or(false)),
                    can_edit,
                )
            }
            Self::Edit {
                view,
                host,
                key,
                processing,
                ..
            } => view.view(&host, &key, *processing),
        }
    }
}

impl From<CosignerSettings> for Box<dyn Setting> {
    fn from(s: CosignerSettings) -> Box<dyn Setting> {
        Box::new(s)
    }
}
