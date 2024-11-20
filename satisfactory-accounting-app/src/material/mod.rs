use yew::{html, AttrValue, Html};

/// Displays a material icon.
pub fn material_icon(name: impl Into<AttrValue>) -> Html {
    html! {
        <span class="material-icons">{name.into()}</span>
    }
}

/// Displays an outlined material icon.
pub fn material_icon_outlined(name: impl Into<AttrValue>) -> Html {
    html! {
        <span class="material-icons-outlined">{name.into()}</span>
    }
}
