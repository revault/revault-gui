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

pub const ALL_SPEND_TX_STATUSES: [SpendTxStatus; 5] = [
    SpendTxStatus::NonFinal,
    SpendTxStatus::Pending,
    SpendTxStatus::Broadcasted,
    SpendTxStatus::Confirmed,
    SpendTxStatus::Deprecated,
];

pub const PROCESSING_SPEND_TX_STATUSES: [SpendTxStatus; 2] =
    [SpendTxStatus::Pending, SpendTxStatus::Broadcasted];

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

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionKind {
    Deposit,
    Unvault,
    Cancel,
    Spend,
    UnvaultEmergency,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct HistoryEventTransaction {
    pub tx: Transaction,
    pub blockheight: u32,
    pub received_time: u32,
    pub blocktime: u32,
    pub kind: TransactionKind,
}

impl HistoryEventTransaction {
    pub fn new(tx: &WalletTransaction, kind: TransactionKind) -> Self {
        Self {
            tx: transaction_from_hex(&tx.hex),
            blockheight: tx.blockheight.unwrap_or(0),
            blocktime: tx.blockheight.unwrap_or(0),
            received_time: tx.received_time,
            kind,
        }
    }
}
