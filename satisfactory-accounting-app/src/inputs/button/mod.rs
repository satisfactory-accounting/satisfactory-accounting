use yew::{
    classes, function_component, html, use_callback, AttrValue, Callback, Classes, Html, Properties,
};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Callback to activate when the button is clicked.
    #[prop_or_default]
    pub onclick: Callback<()>,

    /// Whether the button should be disabled.
    #[prop_or_default]
    pub disabled: bool,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the main button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

/// Simple button with helper for material icons.
#[function_component]
pub fn Button(
    Props {
        children,
        onclick,
        disabled,
        class,
        title,
    }: &Props,
) -> Html {
    let disabled = *disabled;
    let class = classes!("Button", class.clone());
    let onclick = use_callback(onclick.clone(), |_, onclick| onclick.emit(()));

    html! {
        <button {class} {onclick} {disabled} {title}>
            { children.clone() }
        </button>
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct LinkProps {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Callback to activate when the button is clicked.
    #[prop_or_default]
    pub href: Option<AttrValue>,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the main button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,

    /// Html target property for the link.
    #[prop_or_default]
    pub target: Option<AttrValue>,
}

/// Simple button with helper for material icons.
#[function_component]
pub fn LinkButton(
    LinkProps {
        children,
        href,
        class,
        title,
        target,
    }: &LinkProps,
) -> Html {
    let class = classes!("Button", class.clone());

    html! {
        <a {class} {href} {target} {title}>
            { children.clone() }
        </a>
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct UploadProps {
    /// Contents of the button.
    #[prop_or_default]
    pub children: Html,

    /// Extra classes to apply to the button.
    #[prop_or_default]
    pub class: Classes,

    /// Title to set on the button element.
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

/// A button that accepts a file upload.
#[function_component]
pub fn UploadButton(
    UploadProps {
        children,
        class,
        title,
    }: &UploadProps,
) -> Html {
    let class = classes!("Button", class.clone());

    html! {
        <label {class} {title}>
            <input type="file" accept="application/json" />
            {children.clone()}
        </label>
    }
}
