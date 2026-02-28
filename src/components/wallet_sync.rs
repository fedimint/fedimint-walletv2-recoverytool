use bdk_wallet::Wallet;
use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GetUntracked, IntoView, OnAttribute, ReadSignal,
    Set, Show, StoredValue, WriteSignal, component, signal, view,
};

use crate::state::{DestForm, WizardState};
use crate::wallet::{create_wallet, sync_wallet};

#[derive(Clone, Copy, PartialEq)]
enum SyncStatus {
    Idle,
    Syncing,
    Done,
    Error,
}

#[component]
pub fn WalletSync(
    state: ReadSignal<WizardState>,
    set_state: WriteSignal<WizardState>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (network, keys) = match state.get_untracked() {
        WizardState::SyncingWallet { network, keys } => (network, keys),
        _ => unreachable!(),
    };

    let (sync_status, set_sync_status) = signal(SyncStatus::Idle);
    let (balance_sats, set_balance_sats) = signal(0u64);
    let (wallet_handle, set_wallet_handle) = signal(Option::<StoredValue<Wallet>>::None);
    let (trigger_next, set_trigger_next) = signal(false);

    // Handle next transition when triggered
    {
        let network = network.clone();
        Effect::new(move |_| {
            if !trigger_next.get() {
                return;
            }
            set_trigger_next.set(false);

            if let Some(wallet) = wallet_handle.get() {
                set_state.set(WizardState::EnteringDestination {
                    network: network.clone(),
                    wallet,
                    form: DestForm::default(),
                });
            }
        });
    }

    let do_sync = {
        let network = network.clone();
        move || {
            if sync_status.get_untracked() == SyncStatus::Syncing {
                return;
            }

            set_sync_status.set(SyncStatus::Syncing);
            set_error.set(None);

            let network = network.clone();
            let keys = keys.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let result = async {
                    let mut wallet = create_wallet(&keys.descriptor, network.network)
                        .map_err(|e| e.to_string())?;
                    let balance = sync_wallet(&mut wallet, &network.esplora_url)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok::<(Wallet, u64), String>((wallet, balance))
                }
                .await;

                match result {
                    Ok((wallet, balance)) => {
                        set_balance_sats.set(balance);
                        set_wallet_handle.set(Some(StoredValue::new(wallet)));
                        set_sync_status.set(SyncStatus::Done);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Sync failed: {}", e)));
                        set_sync_status.set(SyncStatus::Error);
                    }
                }
            });
        }
    };

    // Auto-start sync on mount
    Effect::new({
        let do_sync = do_sync.clone();
        move |prev_ran: Option<bool>| {
            if prev_ran.is_none() && sync_status.get_untracked() == SyncStatus::Idle {
                do_sync();
            }
            true
        }
    });

    let format_btc = |sats: u64| {
        let btc = sats as f64 / 100_000_000.0;
        format!("{:.8} BTC", btc)
    };

    let is_syncing = move || sync_status.get() == SyncStatus::Syncing;
    let is_done = move || sync_status.get() == SyncStatus::Done;

    view! {
        <div>
            <h5 class="card-title mb-3">"Wallet Sync"</h5>

            <Show when=is_syncing>
                <div class="text-center py-5">
                    <div class="spinner-border text-primary mb-3" role="status">
                        <span class="visually-hidden">"Loading..."</span>
                    </div>
                    <p class="text-muted">"Syncing wallet with blockchain..."</p>
                    <p class="text-muted small">"This may take a moment."</p>
                </div>
            </Show>

            <Show when=is_done>
                <div>
                    <div class="text-center py-4">
                        <div class="balance-label">"Available Balance"</div>
                        <div class="balance-display">
                            {move || format_btc(balance_sats.get())}
                        </div>
                        <div class="text-muted">
                            {move || format!("{} sats", balance_sats.get())}
                        </div>
                    </div>

                    <Show when=move || balance_sats.get() == 0>
                        <div class="warning-box mb-3">
                            <strong>"No funds found. "</strong>
                            "The wallet appears to be empty. Double-check your keys and network selection."
                        </div>
                    </Show>

                    <div class="d-flex justify-content-end">
                        <button
                            type="button"
                            class="btn btn-primary"
                            disabled=move || balance_sats.get() == 0
                            on:click=move |_| set_trigger_next.set(true)
                        >
                            "Next →"
                        </button>
                    </div>
                </div>
            </Show>

            <Show when=move || !is_syncing() && !is_done()>
                <div class="text-center py-4">
                    <p class="text-muted">"Ready to sync wallet with blockchain."</p>
                    <button
                        type="button"
                        class="btn btn-primary"
                        on:click={
                            let do_sync = do_sync.clone();
                            move |_| do_sync()
                        }
                    >
                        "Start Sync"
                    </button>
                </div>
            </Show>
        </div>
    }
}
