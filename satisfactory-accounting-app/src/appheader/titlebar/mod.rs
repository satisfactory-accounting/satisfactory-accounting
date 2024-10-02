use yew::{function_component, html, Html};

/// Displays the app name/title.
#[function_component]
pub fn TitleBar() -> Html {
    html! {
        <div class="TitleBar">
            <h1 class="app-title">{"SATISFACTORY ACCOUNTING"}</h1>
        </div>
    }
}
