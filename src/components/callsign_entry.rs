use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_primitives::{
    label::Label,
    popover::{PopoverContent, PopoverRoot},
};

use crate::types::Callsign;

#[derive(PartialEq, Props, Clone)]
pub struct CallsignEntryProps {
    /// The list of callsigns that can be used as suggestions
    #[props(into, default = use_signal(Vec::new))]
    suggestion_source: ReadOnlySignal<Vec<Callsign>>,
    /// A bi-directional signal with the value of the callsign
    #[props(into, default = use_signal(Callsign::default))]
    value: Signal<Callsign>,
    /// An event handler for when the selected callsign is changed.
    #[props(default = |_| ())]
    on_change: EventHandler<Callsign>,
    /// An event handler that is triggered when the callsign field is
    /// mounted
    #[props(default = |_| ())]
    on_mounted_callsign: EventHandler<Option<Rc<MountedData>>>,
    /// An event handler that is triggered when the name field is
    /// mounted
    #[props(default = |_| ())]
    on_mounted_name: EventHandler<Option<Rc<MountedData>>>,
    /// An optional prefix to prepend to the HTML `id` tags, to avoid
    /// conflicts in the rendered page.
    id_prefix: Option<String>,
    /// An optional prefix to prepend to the labels to better describe
    /// the purpose of this callsign entry, for example you may want to
    /// set this to "Buyer's", so that the user sees "Buyer's Callsign".
    label_prefix: Option<String>,
}

/// [`CallsignEntry`] is a reusable component that offers a simple
/// interface for users to enter callsigns. It can offer suggestions
/// based on a list of pre-known callsigns, and aids in providing the
/// proper formatting.
#[component]
pub fn CallsignEntry(props: CallsignEntryProps) -> Element {
    let mut callsign = props.value;
    let mut suggestion: Signal<Option<Callsign>> = use_signal(|| None);

    let mut callsign_elem: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect(move || {
        props.on_mounted_callsign.call(callsign_elem());
    });
    let mut name_elem: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect(move || {
        props.on_mounted_name.call(name_elem());
    });

    let mut callsign_focussed = use_signal(|| false);
    let mut name_focussed = use_signal(|| false);

    let label_prefix = props
        .label_prefix
        .map(|mut s| {
            s.push(' ');
            s
        })
        .unwrap_or_default();
    let id_prefix = props.id_prefix.unwrap_or_default();

    use_effect(move || {
        // Tell the parent that the entry has changed
        props.on_change.call(callsign());

        // Don't show suggestions if nothing is typed, or not focussed
        if callsign().callsign().is_empty() || !(callsign_focussed() || name_focussed()) {
            suggestion.set(None);
            return;
        }
        // Determine new suggestions
        let sug = props
            .suggestion_source
            .read()
            .clone()
            .into_iter()
            .find(|cs| {
                cs.callsign().starts_with(callsign().callsign())
                    && cs.callsign() != callsign().callsign()
            });
        suggestion.set(sug);
    });

    let accept_suggestion = move || async move {
        if let Some(sug) = suggestion() {
            callsign
                .write()
                .set_callsign(sug.callsign().clone())
                .set_name(sug.name().clone());
            if let Some(name_elem) = name_elem() {
                _ = name_elem.set_focus(true).await;
            }
        }
        suggestion.set(None);
    };

    rsx! {
        PopoverRoot { class: "popover", open: suggestion.read().is_some(),

            div { display: "flex", flex_direction: "row", gap: ".5rem",
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "{id_prefix}callsign", "{label_prefix}Callsign" }

                    input {
                        class: "input",
                        id: "{id_prefix}callsign",
                        value: "{callsign.read().callsign()}",
                        placeholder: "M0ABC",
                        style: "width: 8em",
                        onmounted: move |cx| callsign_elem.set(Some(cx.data())),
                        oninput: move |e| {
                            callsign.write().set_callsign(e.value().trim().to_ascii_uppercase());
                        },
                        onkeydown: move |e| async move {
                            if !e.modifiers().is_empty() {
                                return;
                            }
                            if e.key() == Key::Character(" ".to_string()) {
                                // Focus name box
                                if let Some(name_elem) = name_elem() {
                                    _ = name_elem.set_focus(true).await;
                                }
                            } else if let Key::Character(k) = e.key() {
                                callsign.write().callsign_mut().push_str(&k.to_ascii_uppercase());
                            } else if e.key() == Key::Tab && suggestion.read().is_some() {
                                accept_suggestion().await;
                            }
                        },
                        onfocusin: move |_| callsign_focussed.set(true),
                        onfocusout: move |_| callsign_focussed.set(false),
                    }
                }
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "{id_prefix}name", "{label_prefix}Name" }

                    input {
                        class: "input",
                        id: "{id_prefix}name",
                        value: "{callsign.read().name()}",
                        placeholder: "Jane Smith",
                        onmounted: move |cx| name_elem.set(Some(cx.data())),
                        oninput: move |e| {
                            callsign.write().set_name(e.value());
                        },
                        onkeydown: move |e| async move {
                            if !e.modifiers().is_empty() {
                                return;
                            }
                            if e.key() == Key::Backspace && callsign.read().name().is_empty() {
                                // Focus callsign box
                                if let Some(callsign_elem) = callsign_elem() {
                                    _ = callsign_elem.set_focus(true).await;
                                }
                            } else if e.key() == Key::Tab && suggestion.read().is_some() {
                                accept_suggestion().await;
                            }
                        },
                        onfocusin: move |_| name_focussed.set(true),
                        onfocusout: move |_| name_focussed.set(false),
                    }
                }
            }

            PopoverContent {
                class: "popover-content",
                gap: "0.25rem",
                text_align: "center",
                "data-align": "start",
                if let Some(cs) = suggestion() {
                    "{cs}?"
                } else {
                    "this shouldn't be displayed!"
                }
                span { font_size: "0.6em", font_style: "italic", "Tab to use suggestion" }
            }
        }
    }
}
