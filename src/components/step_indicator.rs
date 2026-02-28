use leptos::prelude::{ClassAttribute, ElementChild, Get, IntoView, ReadSignal, component, view};

use crate::state::WizardState;

#[component]
pub fn StepIndicator(state: ReadSignal<WizardState>) -> impl IntoView {
    view! {
        <div class="step-indicator">
            {(1..=4).map(|num| {
                let step_class = move || {
                    let current = state.get().step_number();
                    if num < current {
                        "step completed"
                    } else if num == current {
                        "step active"
                    } else {
                        "step"
                    }
                };

                let line_class = move || {
                    let current = state.get().step_number();
                    if num < current {
                        "step-line completed"
                    } else {
                        "step-line"
                    }
                };

                view! {
                    {if num > 1 {
                        Some(view! { <div class=line_class></div> })
                    } else {
                        None
                    }}
                    <div class=step_class>{num}</div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
