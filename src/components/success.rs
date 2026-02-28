use bdk_wallet::bitcoin::Network;
use leptos::prelude::{
    ClassAttribute, ElementChild, GetUntracked, IntoView, ReadSignal, component, view,
};

use crate::state::WizardState;

#[component]
pub fn Success(state: ReadSignal<WizardState>) -> impl IntoView {
    let (network, txid) = match state.get_untracked() {
        WizardState::Success { network, txid } => (network, txid),
        _ => unreachable!(),
    };

    let tx_url = match network.network {
        Network::Bitcoin => Some(format!("https://mempool.space/tx/{txid}")),
        Network::Testnet => Some(format!("https://mempool.space/testnet/tx/{txid}")),
        Network::Signet => Some(format!("https://mempool.space/signet/tx/{txid}")),
        _ => None,
    };

    view! {
        <div class="text-center">
            <div class="success-icon mb-3">"✓"</div>
            <p class="text-muted mb-4">"Your funds have been sent successfully."</p>
            {tx_url.map(|url| view! {
                <a
                    href=url
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn btn-primary"
                >
                    "View on mempool.space"
                </a>
            })}
        </div>
    }
}
