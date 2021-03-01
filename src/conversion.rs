use bitcoin::Network;

/// Converter purpose is to give a Conversion from a given amount in satoshis according to its
/// parameters.
pub struct Converter {
    pub unit: Unit,
}

impl Converter {
    pub fn new(bitcoin_network: Network) -> Self {
        let unit = match bitcoin_network {
            Network::Testnet => Unit::TestnetBitcoin,
            Network::Bitcoin => Unit::Bitcoin,
            Network::Regtest => Unit::RegtestBitcoin,
        };
        Self { unit }
    }

    /// converts amount in satoshis to BTC float.
    pub fn converts(&self, amount: u64) -> f64 {
        bitcoin::Amount::from_sat(amount).as_btc()
    }
}

/// Unit is the bitcoin ticker according to the network used.
pub enum Unit {
    TestnetBitcoin,
    RegtestBitcoin,
    Bitcoin,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TestnetBitcoin => write!(f, "tBTC"),
            Self::RegtestBitcoin => write!(f, "rBTC"),
            Self::Bitcoin => write!(f, "BTC"),
        }
    }
}
