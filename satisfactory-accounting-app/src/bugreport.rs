use yew::{html, Html};

pub const ISSUES_PAGE: &'static str = "https://github.com/satisfactory-accounting/satisfactory-accounting/issues";

/// Creates a link with the text "file a bug on GitHub", which points to the github issues page.
pub fn file_a_bug() -> Html {
    html! {
        <a target="_blank" href={ISSUES_PAGE}>{"file a bug on GitHub"}</a>
    }
}
