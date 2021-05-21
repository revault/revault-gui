use crate::revault::Role;

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Previous,
    Role(&'static [Role]),
    DefineStakeholderXpubs(DefineStakeholderXpubs),
    DefineManagerXpubs(DefineManagerXpubs),
    DefineCpfpDescriptor(DefineCpfpDescriptor),
    DefineCoordinator(DefineCoordinator),
    DefineEmergencyAddress(String),
}

#[derive(Debug, Clone)]
pub enum DefineCoordinator {
    HostEdited(String),
    NoiseKeyEdited(String),
}

#[derive(Debug, Clone)]
pub enum DefineCpfpDescriptor {
    ManagerXpub(usize, ParticipantXpub),
    AddXpub,
}

#[derive(Debug, Clone)]
pub enum DefineStakeholderXpubs {
    OurXpubEdited(String),
    StakeholderXpub(usize, ParticipantXpub),
    AddXpub,
}

#[derive(Debug, Clone)]
pub enum DefineManagerXpubs {
    ManagersTreshold(Action),
    SpendingDelay(Action),
    OurXpubEdited(String),
    ManagerXpub(usize, ParticipantXpub),
    CosignerKey(usize, CosignerKey),
    AddXpub,
    AddCosigner,
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

#[derive(Debug, Clone)]
pub enum CosignerKey {
    Delete,
    KeyEdited(String),
}
