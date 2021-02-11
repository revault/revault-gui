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
}
