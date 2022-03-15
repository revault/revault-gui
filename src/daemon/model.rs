use bitcoin::{consensus::encode, hashes::hex::FromHex, OutPoint, Transaction};

pub use revaultd::commands::{
    GetInfoResult, HistoryEvent, HistoryEventKind, ListOnchainTxEntry, ListSpendEntry,
    ListSpendStatus, RevocationTransactions, ServerStatus, ServersStatuses, VaultStatus,
    WalletTransaction,
};
use revaultd::commands::{ListPresignedTxEntry, ListVaultsEntry};

pub type Vault = ListVaultsEntry;

pub fn outpoint(vault: &Vault) -> OutPoint {
    OutPoint::new(vault.txid, vault.vout)
}

pub const DEPOSIT_AND_CURRENT_VAULT_STATUSES: [VaultStatus; 11] = [
    VaultStatus::Funded,
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
    VaultStatus::Active,
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const CURRENT_VAULT_STATUSES: [VaultStatus; 10] = [
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
    VaultStatus::Active,
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const ACTIVE_VAULT_STATUSES: [VaultStatus; 1] = [VaultStatus::Active];

pub const INACTIVE_VAULT_STATUSES: [VaultStatus; 4] = [
    VaultStatus::Funded,
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
];

pub const MOVING_VAULT_STATUSES: [VaultStatus; 6] = [
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const MOVED_VAULT_STATUSES: [VaultStatus; 4] = [
    VaultStatus::Canceled,
    VaultStatus::EmergencyVaulted,
    VaultStatus::UnvaultEmergencyVaulted,
    VaultStatus::Spent,
];

pub type SpendTxStatus = ListSpendStatus;

pub type VaultTransactions = ListOnchainTxEntry;
pub type VaultPresignedTransactions = ListPresignedTxEntry;

pub fn transaction_from_hex(hex: &str) -> Transaction {
    let bytes = Vec::from_hex(&hex).unwrap();
    encode::deserialize::<Transaction>(&bytes).unwrap()
}

pub type SpendTx = ListSpendEntry;

pub const ALL_HISTORY_EVENTS: [HistoryEventKind; 3] = [
    HistoryEventKind::Cancel,
    HistoryEventKind::Deposit,
    HistoryEventKind::Spend,
];
