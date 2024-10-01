use yew::{function_component, html, Html, Properties};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Left items in the menu.
    #[prop_or_default]
    pub left: Html,
    /// Right items in the menu.
    #[prop_or_default]
    pub right: Html,
}

/// Displays the main Yew menu bar.
#[function_component]
pub fn MenuBar(Props { left, right }: &Props) -> Html {
    html! {
        <div class="MenuBar">
            <div class="flex-section">
                { left.clone() }
            </div>
            <div class="flex-section">
                { right.clone() }
            </div>
        </div>
    }
}
