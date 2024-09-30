use yew::{function_component, html, Callback, Html, Properties};

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct Props {
    /// Callback to perform undo, if undo is available.
    undo: Option<Callback<()>>,
    /// Callback to perform redo, if redo is available.
    redo: Option<Callback<()>>,
}

/// Displays the main Yew menu bar.
#[function_component]
pub fn MenuBar(Props { undo, redo }: &Props) -> Html {
    html! {
        <div class="MenuBar">
            <div class="flex-section">

            </div>
        </div>
    }
}
