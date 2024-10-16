use yew::{html, AttrValue, Html};

/// Displays a material icon.
pub fn material_icon(name: impl Into<AttrValue>) -> Html {
    html! {
        <span class="material-icons">{name.into()}</span>
    }
}

/// Displays a material icon from the outlined.
pub fn material_icon_outline(name: impl Into<AttrValue>) -> Html {
    html! {
        <span class="material-icons-outlined">{name.into()}</span>
    }
}
