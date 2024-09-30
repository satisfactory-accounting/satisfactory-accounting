use yew::{html, AttrValue, Html};

/// Displays a material icon.
pub fn material_icon(name: impl Into<AttrValue>) -> Html {
    html! {
        <span class="material-icons">{name.into()}</span>
    }
}
