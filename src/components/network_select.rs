use bdk_wallet::bitcoin::Network;
use leptos::prelude::{
    ClassAttribute, ElementChild, Get, GetUntracked, GlobalAttributes, IntoView, OnAttribute,
    PropAttribute, ReadSignal, Set, Update, WriteSignal, component, event_target_value, signal,
    view,
};

use crate::state::{KeysForm, NetworkConfig, WizardState, esplora_url_for_network};

fn validate_url(s: &str) -> bool {
    let trimmed = s.trim();
    !trimmed.is_empty() && (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
}

#[component]
pub fn NetworkSelect(
    state: ReadSignal<WizardState>,
    set_state: WriteSignal<WizardState>,
) -> impl IntoView {
    let initial_form = match state.get_untracked() {
        WizardState::SelectingNetwork { form } => form,
        _ => unreachable!(),
    };

    let (local_form, set_local_form) = signal(initial_form);

    let networks = vec![
        (Network::Bitcoin, "Mainnet"),
        (Network::Testnet, "Testnet"),
        (Network::Signet, "Signet"),
        (Network::Regtest, "Regtest"),
    ];

    let select_network = move |network: Network| {
        set_local_form.update(|f| {
            f.network = network;
            f.esplora_url = esplora_url_for_network(network).to_string();
        });
    };

    let on_next = move |_| {
        let f = local_form.get();
        set_state.set(WizardState::EnteringKeys {
            network: NetworkConfig {
                network: f.network,
                esplora_url: f.esplora_url,
            },
            form: KeysForm::default(),
        });
    };

    view! {
        <div>
            <h5 class="card-title mb-3">"Select Network"</h5>

            <div class="mb-4">
                <div class="btn-group w-100" role="group">
                    {networks.into_iter().map(|(network, label)| {
                        let is_active = move || local_form.get().network == network;
                        let btn_class = move || {
                            if is_active() {
                                "btn btn-primary network-btn active"
                            } else {
                                "btn btn-outline-primary network-btn"
                            }
                        };

                        view! {
                            <button
                                type="button"
                                class=btn_class
                                on:click=move |_| select_network(network)
                            >
                                {label}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div class="mb-4 mt-3">
                <input
                    type="text"
                    class=move || {
                        let url = local_form.get().esplora_url.clone();
                        if validate_url(&url) {
                            "form-control is-valid"
                        } else {
                            "form-control is-invalid"
                        }
                    }
                    id="esplora-url"
                    spellcheck="false"
                    autocomplete="off"
                    prop:value=move || local_form.get().esplora_url.clone()
                    on:input=move |ev| {
                        let value = event_target_value(&ev).trim().to_string();
                        set_local_form.update(|f| f.esplora_url = value);
                    }
                />
                <div class="form-text">
                    "The Esplora API endpoint to use for blockchain queries."
                </div>
            </div>

            <div class="d-flex justify-content-end">
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=on_next
                >
                    "Next →"
                </button>
            </div>
        </div>
    }
}
