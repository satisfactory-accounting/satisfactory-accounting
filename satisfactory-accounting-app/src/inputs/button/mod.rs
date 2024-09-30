use yew::{classes, function_component, html, use_callback, Callback, Classes, Html, Properties};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Contents of the button.
    #[prop_or_default]
    children: Html,

    /// Callback to activate when the button is clicked.
    #[prop_or_default]
    onclick: Option<Callback<()>>,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    class: Classes,
}

/// Simple button with helper for material icons.
#[function_component]
pub fn Button(
    Props {
        children,
        onclick,
        class,
    }: &Props,
) -> Html {
    let disabled = onclick.is_none();
    let class = classes!(
        "Button",
        class.clone()
    );
    let onclick = use_callback(onclick.clone(), |_, onclick| {
        if let Some(onclick) = onclick {
            onclick.emit(())
        }
    });

    html! {
        <button {class} {onclick} {disabled}>
            { children.clone() }
        </button>
    }
}
