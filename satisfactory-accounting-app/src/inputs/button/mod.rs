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
    pub onclick: Option<Callback<()>>,

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
        class,
        title,
    }: &Props,
) -> Html {
    let disabled = onclick.is_none();
    let class = classes!("Button", class.clone());
    let onclick = use_callback(onclick.clone(), |_, onclick| {
        if let Some(onclick) = onclick {
            onclick.emit(())
        }
    });

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
