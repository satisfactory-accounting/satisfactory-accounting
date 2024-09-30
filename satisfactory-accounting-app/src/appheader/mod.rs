use yew::{function_component, html, Html};

use menubar::MenuBar;
use titlebar::TitleBar;

mod menubar;
mod titlebar;

/// Displays the App header including titlebar and menubar.
#[function_component]
pub fn AppHeader(props: &menubar::Props) -> Html {
    html! {
        <div class="AppHeader">
            <TitleBar />
            <MenuBar ..props.clone() />
        </div>
    }
}
