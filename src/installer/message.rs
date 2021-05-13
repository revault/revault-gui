use crate::revault::Role;

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Previous,
    Role(&'static [Role]),
    DefineStakeholderXpubs(DefineStakeholderXpubs),
    DefineManagerXpubs(DefineManagerXpubs),
}

#[derive(Debug, Clone)]
pub enum DefineStakeholderXpubs {
    OurXpubEdited(String),
    StakeholderXpub(usize, ParticipantXpub),
    AddXpub,
}

#[derive(Debug, Clone)]
pub enum DefineManagerXpubs {
    OurXpubEdited(String),
    ManagerXpub(usize, ParticipantXpub),
    CosignerKey(usize, CosignerKey),
    AddXpub,
    AddCosigner,
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
