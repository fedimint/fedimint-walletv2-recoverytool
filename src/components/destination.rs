use std::str::FromStr;

use bdk_wallet::bitcoin::{Address, Network};
use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GetUntracked, GlobalAttributes, IntoView,
    OnAttribute, PropAttribute, ReadSignal, Set, Show, Update, UpdateValue, WriteSignal, component,
    event_target_value, signal, view,
};

use crate::state::WizardState;
use crate::wallet::{broadcast_tx, build_sweep_tx};

fn parse_address(s: &str, network: Network) -> Result<Address, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("Please enter a destination address".to_string());
    }
    Address::from_str(trimmed)
        .map_err(|e| format!("Invalid address: {e}"))?
        .require_network(network)
        .map_err(|e| format!("Address network mismatch: {e}"))
}

#[component]
pub fn Destination(
    state: ReadSignal<WizardState>,
    set_state: WriteSignal<WizardState>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (network, wallet, initial_form) = match state.get_untracked() {
        WizardState::EnteringDestination {
            network,
            wallet,
            form,
            ..
        } => (network, wallet, form),
        _ => unreachable!(),
    };

    let (local_form, set_local_form) = signal(initial_form);
    let (broadcasting, set_broadcasting) = signal(false);
    let (trigger_broadcast, set_trigger_broadcast) = signal(false);

    // Handle broadcast when triggered
    {
        let network = network.clone();
        Effect::new(move |_| {
            if !trigger_broadcast.get() {
                return;
            }
            set_trigger_broadcast.set(false);

            let current_form = local_form.get_untracked();

            // Validate destination address
            let destination =
                match parse_address(&current_form.destination_address, network.network) {
                    Ok(addr) => addr,
                    Err(e) => {
                        set_error.set(Some(e));
                        return;
                    }
                };

            if current_form.fee_rate == 0 {
                set_error.set(Some("Fee rate must be at least 1 sat/vB".to_string()));
                return;
            }

            set_broadcasting.set(true);
            set_error.set(None);

            let network = network.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Build PSBT using the stored wallet (synchronous, mutable access)
                let psbt_result = wallet
                    .try_update_value(|w| build_sweep_tx(w, destination, current_form.fee_rate));

                let result = match psbt_result {
                    Some(Ok(psbt)) => broadcast_tx(&network.esplora_url, &psbt)
                        .await
                        .map_err(|e| e.to_string()),
                    Some(Err(e)) => Err(e.to_string()),
                    None => Err("Wallet not available".to_string()),
                };

                match result {
                    Ok(txid) => {
                        set_state.set(WizardState::Success { network, txid });
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Broadcast failed: {e}")));
                    }
                }
                set_broadcasting.set(false);
            });
        });
    }

    let address_class = {
        let network = network.clone();
        move || {
            let f = local_form.get();
            if f.destination_address.trim().is_empty() {
                "form-control"
            } else if parse_address(&f.destination_address, network.network).is_ok() {
                "form-control is-valid"
            } else {
                "form-control is-invalid"
            }
        }
    };

    let is_valid = {
        let network = network.clone();
        move || {
            let f = local_form.get();
            parse_address(&f.destination_address, network.network).is_ok() && f.fee_rate > 0
        }
    };

    view! {
        <div>
            <h5 class="card-title mb-3">"Destination"</h5>

            <Show when=move || broadcasting.get()>
                <div class="text-center py-5">
                    <div class="spinner-border text-primary mb-3" role="status">
                        <span class="visually-hidden">"Broadcasting..."</span>
                    </div>
                    <p class="text-muted">"Broadcasting transaction..."</p>
                </div>
            </Show>

            <Show when=move || !broadcasting.get()>
                <div class="mb-4">
                    <label for="destination-address" class="form-label">"Address"</label>
                    <input
                        type="text"
                        class=address_class
                        id="destination-address"
                        placeholder="Enter Bitcoin address..."
                        spellcheck="false"
                        autocomplete="off"
                        prop:value=move || local_form.get().destination_address.clone()
                        on:input=move |ev| {
                            let value = event_target_value(&ev).trim().to_string();
                            set_local_form.update(|f| f.destination_address = value);
                        }
                    />
                </div>

                <div class="mb-4">
                    <label for="fee-rate" class="form-label">"Fee Rate (sat/vB)"</label>
                    <input
                        type="number"
                        class="form-control"
                        id="fee-rate"
                        min="1"
                        prop:value=move || local_form.get().fee_rate.to_string()
                        on:input=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<u64>() {
                                set_local_form.update(|f| f.fee_rate = value);
                            }
                        }
                    />
                </div>

                <div class="d-flex justify-content-end">
                    <button
                        type="button"
                        class="btn btn-danger"
                        disabled=move || !is_valid()
                        on:click=move |_| set_trigger_broadcast.set(true)
                    >
                        "Confirm & Broadcast"
                    </button>
                </div>
            </Show>
        </div>
    }
}
