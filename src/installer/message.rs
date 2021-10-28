use std::path::PathBuf;

use super::Error;
use crate::revault::Role;

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced_native::Event),
    Exit(PathBuf),
    Next,
    Previous,
    Install,
    Installed(Result<PathBuf, Error>),
    Role(&'static [Role]),
    PrivateNoiseKey(String),
    DefineStakeholderXpubs(DefineStakeholderXpubs),
    DefineManagerXpubs(DefineManagerXpubs),
    DefineCpfpDescriptor(DefineCpfpDescriptor),
    DefineCoordinator(DefineCoordinator),
    DefineEmergencyAddress(String),
    DefineWatchtowers(DefineWatchtowers),
    DefineCosigners(usize, DefineCosigner),
    DefineBitcoind(DefineBitcoind),
}

#[derive(Debug, Clone)]
pub enum DefineBitcoind {
    NetworkEdited(bitcoin::Network),
    CookiePathEdited(String),
    AddressEdited(String),
}

#[derive(Debug, Clone)]
pub enum DefineCosigner {
    HostEdited(String),
    NoiseKeyEdited(String),
}

#[derive(Debug, Clone)]
pub enum DefineWatchtowers {
    EditWatchtower(usize, DefineWatchtower),
    AddWatchtower,
}

#[derive(Debug, Clone)]
pub enum DefineWatchtower {
    HostEdited(String),
    NoiseKeyEdited(String),
    Delete,
}

#[derive(Debug, Clone)]
pub enum DefineCoordinator {
    HostEdited(String),
    NoiseKeyEdited(String),
}

#[derive(Debug, Clone)]
pub enum DefineCpfpDescriptor {
    ManagerXpub(usize, String),
}

#[derive(Debug, Clone)]
pub enum DefineStakeholderXpubs {
    OurXpubEdited(String),
    StakeholderXpub(usize, ParticipantXpub),
    AddXpub,
}

#[derive(Debug, Clone)]
pub enum DefineManagerXpubs {
    ManagersThreshold(Action),
    SpendingDelay(Action),
    OurXpubEdited(String),
    ManagerXpub(usize, ParticipantXpub),
    CosignersEnabled(bool),
    CosignerKey(usize, String),
    AddXpub,
}

#[derive(Debug, Clone)]
pub enum Action {
    Increment,
    Decrement,
}

#[derive(Debug, Clone)]
pub enum ParticipantXpub {
    Delete,
    XpubEdited(String),
}
