mod components;
mod state;
mod wallet;

use leptos::prelude::{
    ClassAttribute, ElementChild, Get, IntoAny, IntoView, Show, component, signal, view,
};

use crate::components::{Destination, KeyInput, NetworkSelect, StepIndicator, Success, WalletSync};
use crate::state::WizardState;

#[component]
fn App() -> impl IntoView {
    let (state, set_state) = signal(WizardState::default());
    let (error, set_error) = signal(Option::<String>::None);

    view! {
        <div class="wizard-container">
            <header class="text-center mb-4">
                <h1 class="h3">"Fedimint Wallet Recovery"</h1>
            </header>

            <StepIndicator state=state />

            <Show when=move || error.get().is_some()>
                <div class="error-box mb-3">
                    <strong>"Error: "</strong>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="card">
                <div class="card-body">
                    {move || match state.get() {
                        WizardState::SelectingNetwork { .. } => view! {
                            <NetworkSelect state=state set_state=set_state />
                        }.into_any(),
                        WizardState::EnteringKeys { .. } => view! {
                            <KeyInput
                                state=state
                                set_state=set_state
                                set_error=set_error
                            />
                        }.into_any(),
                        WizardState::SyncingWallet { .. } => view! {
                            <WalletSync
                                state=state
                                set_state=set_state
                                set_error=set_error
                            />
                        }.into_any(),
                        WizardState::EnteringDestination { .. } => view! {
                            <Destination
                                state=state
                                set_state=set_state
                                set_error=set_error
                            />
                        }.into_any(),
                        WizardState::Success { .. } => view! {
                            <Success state=state />
                        }.into_any(),
                    }}
                </div>
            </div>

            <footer class="text-center mt-4">
                <small class="text-muted">
                    "Private keys are processed locally in your browser. "
                    "This tool does not transmit keys to any server."
                </small>
            </footer>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    // Remove the loading spinner now that wasm is ready
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("loading"))
    {
        el.remove();
    }
    leptos::mount::mount_to_body(App);
}
