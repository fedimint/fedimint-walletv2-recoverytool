use std::str::FromStr;

use bitcoin::PrivateKey;
use bitcoin::secp256k1::PublicKey;
use leptos::ev::MouseEvent;
use leptos::prelude::{
    ClassAttribute, ElementChild, Get, GetUntracked, GlobalAttributes, IntoView, OnAttribute,
    PropAttribute, ReadSignal, Set, Update, WriteSignal, component, event_target_value, signal,
    view,
};

use crate::state::{KeysConfig, WizardState};
use crate::wallet::{bft_threshold, build_descriptor, create_wallet};

fn validate_private_key(s: &str) -> bool {
    let trimmed = s.trim();
    !trimmed.is_empty() && PrivateKey::from_wif(trimmed).is_ok()
}

fn validate_public_key(s: &str) -> bool {
    let trimmed = s.trim();
    !trimmed.is_empty() && PublicKey::from_str(trimmed).is_ok()
}

#[component]
pub fn KeyInput(
    state: ReadSignal<WizardState>,
    set_state: WriteSignal<WizardState>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (network, initial_form) = match state.get_untracked() {
        WizardState::EnteringKeys { network, form } => (network, form),
        _ => unreachable!(),
    };

    let (local_form, set_local_form) = signal(initial_form);

    let adjust_count = move |delta: i32| {
        set_local_form.update(|f| {
            let new_count = (f.guardian_count as i32 + delta).clamp(4, 20) as usize;
            if new_count != f.guardian_count {
                f.guardian_count = new_count;
                let threshold = bft_threshold(new_count);
                f.private_keys.resize(threshold, String::new());
                let public_count = new_count.saturating_sub(threshold);
                f.public_keys.resize(public_count, String::new());
            }
        });
    };

    let validate_and_next = move |_ev: MouseEvent| {
        let current_form = local_form.get();
        let threshold = current_form.threshold();

        let private_count = current_form
            .private_keys
            .iter()
            .filter(|k| validate_private_key(k))
            .count();

        let public_count = current_form
            .public_keys
            .iter()
            .filter(|k| validate_public_key(k))
            .count();

        let total = private_count + public_count;

        if private_count < threshold {
            set_error.set(Some(format!(
                "Need at least {} valid private keys (threshold), got {}",
                threshold, private_count
            )));
            return;
        }

        if total != current_form.guardian_count {
            set_error.set(Some(format!(
                "Total valid keys ({}) must equal guardian count ({})",
                total, current_form.guardian_count
            )));
            return;
        }

        let descriptor = build_descriptor(
            current_form.threshold(),
            &current_form.private_keys,
            &current_form.public_keys,
        );

        match create_wallet(&descriptor, network.network) {
            Ok(_) => {
                set_error.set(None);
                set_state.set(WizardState::SyncingWallet {
                    network: network.clone(),
                    keys: KeysConfig { descriptor },
                });
            }
            Err(e) => {
                set_error.set(Some(format!("Invalid keys: {e}")));
            }
        }
    };

    view! {
        <div>
            <h5 class="card-title mb-3">"Enter Guardian Keys"</h5>

            <div class="mb-4">
                <div class="d-flex align-items-center justify-content-center gap-3 my-3">
                    <button
                        type="button"
                        class="btn btn-outline-secondary btn-lg px-3"
                        on:click=move |_| adjust_count(-1)
                    >
                        "-"
                    </button>
                    <span class="guardian-count-display">
                        {move || local_form.get().guardian_count}
                    </span>
                    <button
                        type="button"
                        class="btn btn-outline-secondary btn-lg px-3"
                        on:click=move |_| adjust_count(1)
                    >
                        "+"
                    </button>
                </div>
                <div class="text-center text-muted">
                    "Number of Guardians"
                </div>
            </div>

            <div class="mb-4">
                <div class="form-text mb-2">
                    "Enter at least " {move || local_form.get().threshold()} " private keys to sign the transaction."
                </div>
                {move || {
                    let threshold = local_form.get().threshold();
                    (0..threshold).map(|i| {
                        let key_value = move || local_form.get().private_keys.get(i).cloned().unwrap_or_default();
                        let input_class = move || {
                            let val = key_value();
                            if val.trim().is_empty() {
                                "form-control"
                            } else if validate_private_key(&val) {
                                "form-control is-valid"
                            } else {
                                "form-control is-invalid"
                            }
                        };
                        view! {
                            <div class="key-input-group">
                                <input
                                    type="text"
                                    class=input_class
                                    placeholder="Enter WIF private key..."
                                    spellcheck="false"
                                    autocomplete="off"
                                    prop:value=key_value
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev).trim().to_string();
                                        set_local_form.update(|f| {
                                            if let Some(key) = f.private_keys.get_mut(i) {
                                                *key = value;
                                            }
                                        });
                                    }
                                />
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>

            <div class="mb-4">
                <div class="form-text mb-2">
                    "Enter the remaining " {move || local_form.get().guardian_count.saturating_sub(local_form.get().threshold())}
                    " public keys of the other guardians."
                </div>
                {move || {
                    let current = local_form.get();
                    let public_count = current.guardian_count.saturating_sub(current.threshold());
                    (0..public_count).map(|i| {
                        let key_value = move || local_form.get().public_keys.get(i).cloned().unwrap_or_default();
                        let input_class = move || {
                            let val = key_value();
                            if val.trim().is_empty() {
                                "form-control"
                            } else if validate_public_key(&val) {
                                "form-control is-valid"
                            } else {
                                "form-control is-invalid"
                            }
                        };
                        view! {
                            <div class="key-input-group">
                                <input
                                    type="text"
                                    class=input_class
                                    placeholder="Enter hex public key..."
                                    spellcheck="false"
                                    autocomplete="off"
                                    prop:value=key_value
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev).trim().to_string();
                                        set_local_form.update(|f| {
                                            if let Some(key) = f.public_keys.get_mut(i) {
                                                *key = value;
                                            }
                                        });
                                    }
                                />
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>

            <div class="d-flex justify-content-end">
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=validate_and_next
                >
                    "Next →"
                </button>
            </div>
        </div>
    }
}
