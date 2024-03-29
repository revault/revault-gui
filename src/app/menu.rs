#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Menu {
    History,
    Deposit,
    Emergency,
    Home,
    Send,
    CreateSpend,
    ImportSpend,
    CreateVaults,
    RevaultVaults,
    DelegateFunds,
    Settings,
    Vaults(VaultsMenu),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultsMenu {
    Current,
    Moving,
    Moved,
}
