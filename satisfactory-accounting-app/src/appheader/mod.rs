use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::{function_component, html, use_callback, Callback, Html, Properties};

use menubar::MenuBar;
use titlebar::TitleBar;

use crate::app::DatabaseChoice;
use crate::inputs::button::{Button, LinkButton};
use crate::material::material_icon;
use crate::user_settings::{
    use_user_settings, use_user_settings_dispatcher, use_user_settings_window,
};

mod menubar;
mod titlebar;

#[derive(Debug, Clone)]
pub struct DatabaseChoiceShallowEq(DatabaseChoice);

impl From<DatabaseChoice> for DatabaseChoiceShallowEq {
    fn from(value: DatabaseChoice) -> Self {
        Self(value)
    }
}

impl IntoPropValue<DatabaseChoiceShallowEq> for DatabaseChoice {
    fn into_prop_value(self) -> DatabaseChoiceShallowEq {
        self.into()
    }
}

impl IntoPropValue<DatabaseChoiceShallowEq> for &DatabaseChoice {
    fn into_prop_value(self) -> DatabaseChoiceShallowEq {
        self.clone().into()
    }
}

impl PartialEq for DatabaseChoiceShallowEq {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (DatabaseChoice::Standard(lhs), DatabaseChoice::Standard(rhs)) => lhs == rhs,
            // No need to do deep comparisons of custom database choices because we just report
            // those as "Custom".
            (DatabaseChoice::Custom(_), DatabaseChoice::Custom(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct Props {
    /// Name of the currently selected database.
    pub db_choice: DatabaseChoiceShallowEq,

    /// Callback to open the world-chooser.
    pub on_choose_world: Callback<()>,
    /// Callback to perform undo, if undo is available.
    pub on_undo: Option<Callback<()>>,
    /// Callback to perform redo, if redo is available.
    pub on_redo: Option<Callback<()>>,
    /// Callback to open the database-chooser.
    pub on_choose_db: Callback<()>,
}

/// Displays the App header including titlebar and menubar.
#[function_component]
pub fn AppHeader(
    Props {
        db_choice: DatabaseChoiceShallowEq(db_choice),
        on_choose_world,
        on_undo,
        on_redo,
        on_choose_db,
    }: &Props,
) -> Html {
    let hide_empty = use_user_settings().hide_empty_balances;

    let settings_dispatcher = use_user_settings_dispatcher();
    let on_toggle_empty = use_callback(settings_dispatcher, |(), settings_dispatcher| {
        settings_dispatcher.toggle_hide_empty_balances();
    });

    let settings_window_dispatcher = use_user_settings_window();
    let on_settings = use_callback(
        settings_window_dispatcher,
        |(), settings_window_dispatcher| settings_window_dispatcher.toggle_window(),
    );

    let left = html! {
        <>
            <Button title="Choose World" onclick={on_choose_world}>
                {material_icon("folder_open")}
            </Button>
            <Button title="Undo" onclick={on_undo}>
                {material_icon("undo")}
            </Button>
            <Button title="Redo" onclick={on_redo}>
                {material_icon("redo")}
            </Button>
            <Button title="Choose Database" onclick={on_choose_db}>
                {material_icon("factory")}
                <span>{db_name(db_choice)}</span>
            </Button>
            <Button class="hide-empty-button" title="Hide Empty Balances" onclick={on_toggle_empty}>
                {material_icon("exposure_zero")}
                if hide_empty {
                    {material_icon("visibility_off")}
                } else {
                    {material_icon("visibility")}
                }
            </Button>
        </>
    };

    let right = html! {
        <>
            <Button title="Settings" onclick={on_settings}>
                {material_icon("settings")}
            </Button>
            <LinkButton title="Bug Report" target="_blank"
                href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues">
                {material_icon("bug_report")}
            </LinkButton>
        </>
    };

    html! {
        <div class="AppHeader">
            <TitleBar />
            <MenuBar {left} {right} />
        </div>
    }
}

/// Get a string representing the name of this database choice for the database chooser button.
fn db_name(db_choice: &DatabaseChoice) -> Cow<'static, str> {
    match db_choice {
        DatabaseChoice::Standard(version) => {
            if version.is_deprecated() {
                Cow::Owned(format!("{version} \u{2013} Update Available!"))
            } else {
                Cow::Borrowed(version.name())
            }
        }
        DatabaseChoice::Custom(_) => Cow::Borrowed("Custom"),
    }
}
