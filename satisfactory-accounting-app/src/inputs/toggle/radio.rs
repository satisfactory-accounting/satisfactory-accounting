use yew::{function_component, html, AttrValue, Callback, Html, MouseEvent, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Whether the radio is currently checked.
    pub checked: bool,
    /// Callback for when the radio button is clicked.
    pub onclick: Callback<MouseEvent>,
    /// Name to use for this radio button.
    #[prop_or_default]
    pub name: Option<AttrValue>,
}

/// Displays a radio button using a material icon in place of the default display type.
#[function_component]
pub fn MaterialRadio(&Props { checked, ref name, ref onclick }: &Props) -> Html {
    html! {
        <div class="MaterialToggle radio">
            <input type="radio" {name} {checked} {onclick} />
            <span class="hidden-input-display material-icons">
                if checked {
                    {"radio_button_checked"}
                } else {
                    {"radio_button_unchecked"}
                }
            </span>
        </div>
    }
}
