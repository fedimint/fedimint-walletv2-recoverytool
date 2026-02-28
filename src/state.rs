use bdk_wallet::Wallet;
use bdk_wallet::bitcoin::{Network, Txid};
use leptos::prelude::StoredValue;

use crate::wallet::bft_threshold;

/// Immutable config produced after selecting network
#[derive(Clone)]
pub struct NetworkConfig {
    pub network: Network,
    pub esplora_url: String,
}

/// Immutable config produced after entering keys
#[derive(Clone)]
pub struct KeysConfig {
    pub descriptor: String,
}

/// Mutable form state during network selection
#[derive(Clone)]
pub struct NetworkForm {
    pub network: Network,
    pub esplora_url: String,
}

impl Default for NetworkForm {
    fn default() -> Self {
        Self {
            network: Network::Bitcoin,
            esplora_url: esplora_url_for_network(Network::Bitcoin).to_string(),
        }
    }
}

/// Mutable form state during key input
#[derive(Clone)]
pub struct KeysForm {
    pub guardian_count: usize,
    pub private_keys: Vec<String>,
    pub public_keys: Vec<String>,
}

impl KeysForm {
    pub fn new(guardian_count: usize) -> Self {
        let threshold = bft_threshold(guardian_count);
        let public_count = guardian_count.saturating_sub(threshold);
        Self {
            guardian_count,
            private_keys: vec![String::new(); threshold],
            public_keys: vec![String::new(); public_count],
        }
    }

    pub fn threshold(&self) -> usize {
        bft_threshold(self.guardian_count)
    }
}

impl Default for KeysForm {
    fn default() -> Self {
        Self::new(4)
    }
}

/// Mutable form state during destination entry
#[derive(Clone, Default)]
pub struct DestForm {
    pub destination_address: String,
    pub fee_rate: u64,
}

/// Progressive wizard state with typed variants
#[derive(Clone)]
pub enum WizardState {
    SelectingNetwork {
        form: NetworkForm,
    },
    EnteringKeys {
        network: NetworkConfig,
        form: KeysForm,
    },
    SyncingWallet {
        network: NetworkConfig,
        keys: KeysConfig,
    },
    EnteringDestination {
        network: NetworkConfig,
        wallet: StoredValue<Wallet>,
        form: DestForm,
    },
    Success {
        network: NetworkConfig,
        txid: Txid,
    },
}

impl Default for WizardState {
    fn default() -> Self {
        WizardState::SelectingNetwork {
            form: NetworkForm::default(),
        }
    }
}

impl WizardState {
    pub fn step_number(&self) -> usize {
        match self {
            WizardState::SelectingNetwork { .. } => 1,
            WizardState::EnteringKeys { .. } => 2,
            WizardState::SyncingWallet { .. } => 3,
            WizardState::EnteringDestination { .. } => 4,
            WizardState::Success { .. } => 5,
        }
    }
}

/// Get the default esplora URL for a network
pub fn esplora_url_for_network(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "https://mempool.space/api",
        Network::Testnet => "https://mempool.space/testnet/api",
        Network::Signet => "https://mempool.space/signet/api",
        Network::Regtest => "http://localhost:3003",
        _ => "https://mempool.space/api",
    }
}
