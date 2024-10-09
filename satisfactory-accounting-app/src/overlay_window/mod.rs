use log::warn;
use web_sys::Element;
use yew::{
    classes, create_portal, function_component, html, use_effect_with, use_memo, AttrValue,
    Callback, Classes, Html, Properties,
};

use crate::inputs::button::Button;
use crate::material::material_icon;

pub mod controller;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Window title, shown in a row with the close button.
    pub title: AttrValue,
    /// Content to render in the window.
    #[prop_or_default]
    pub children: Html,
    /// Extra classes to apply to the window.
    #[prop_or_default]
    pub class: Classes,

    /// Callback for when the window is closed.
    #[prop_or_default]
    pub on_close: Callback<()>,
}

/// Draws an overlay window relative to its parent.
#[function_component]
pub fn OverlayWindow(props: &Props) -> Html {
    let host = use_memo((), |()| {
        gloo::utils::document()
            .create_element("div")
            .expect("Unable to create element")
    });
    let host = Element::clone(&*host);

    use_effect_with(host.clone(), |host| {
        let modal_host = gloo::utils::document()
            .get_element_by_id("modal-host")
            .expect("Missing Modal Host");

        if let Err(e) = modal_host.append_child(host) {
            warn!("Unable to attach modal host element: {e:?}")
        }

        let host = host.clone();
        move || {
            if let Err(e) = modal_host.remove_child(&host) {
                warn!("Unable to detach modal host element: {e:?}")
            }
        }
    });

    html! {
        { create_portal(overlay_contents(props), host) }
    }
}

/// Renders the actual overlay contents.
fn overlay_contents(
    Props {
        title,
        children,
        class,
        on_close,
    }: &Props,
) -> Html {
    html! {
        <div class={classes!("OverlayWindow", class.clone())}>
            <section class="window-title">
                <h1>{title}</h1>
                <Button title="Close" class="red" onclick={on_close}>
                    {material_icon("close")}
                </Button>
            </section>
            <section class="window-content-wrapper">
                <div class="window-content">
                    {children.clone()}
                </div>
            </section>
        </div>
    }
}
