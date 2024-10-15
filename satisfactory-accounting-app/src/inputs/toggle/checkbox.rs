use yew::{function_component, html, Callback, Html, MouseEvent, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Whether the radio is currently checked.
    pub checked: bool,
    /// Callback for when the radio button is clicked.
    pub onclick: Callback<MouseEvent>,
}

/// Displays a radio button using a material icon in place of the default display type.
#[function_component]
pub fn MaterialCheckbox(
    &Props {
        checked,
        ref onclick,
    }: &Props,
) -> Html {
    html! {
        <div class="MaterialToggle checkbox">
            <input type="checkbox" {checked} {onclick} />
            <span class="hidden-input-display material-icons">
                if checked {
                    {"check_box"}
                } else {
                    {"check_box_outline_blank"}
                }
            </span>
        </div>
    }
}
