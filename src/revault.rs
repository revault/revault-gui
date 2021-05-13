#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Manager,
    Stakeholder,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::Manager => "Manager",
                Role::Stakeholder => "Stakeholder",
            }
        )
    }
}

impl Role {
    pub const ALL: [Role; 2] = [Role::Manager, Role::Stakeholder];
    pub const MANAGER_ONLY: [Role; 1] = [Role::Manager];
    pub const STAKEHOLDER_ONLY: [Role; 1] = [Role::Stakeholder];
    pub const STAKEHOLDER_AND_MANAGER: [Role; 2] = [Role::Stakeholder, Role::Manager];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionKind {
    Emergency,
    EmergencyUnvault,
    Unvault,
    Cancel,
    Spend,
}
