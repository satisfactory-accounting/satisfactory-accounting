use yew::{function_component, html, Html};

/// Displays the app name/title.
#[function_component]
pub fn TitleBar() -> Html {
    html! {
        <div class="TitleBar">
            <div class="app-title">{"SATISFACTORY ACCOUNTING"}</div>
        </div>
    }
}
