use yew::{function_component, html, AttrValue, Callback, Html, Properties};

use menubar::MenuBar;
use titlebar::TitleBar;

use crate::inputs::button::{Button, LinkButton};
use crate::material::material_icon;

mod menubar;
mod titlebar;

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct Props {
    /// Name of the currently selected database.
    pub dbname: AttrValue,
    /// Whether to hide empty balances.
    pub hide_empty: bool,

    /// Callback to open the world-chooser.
    pub choose_world: Callback<()>,
    /// Callback to perform undo, if undo is available.
    pub undo: Option<Callback<()>>,
    /// Callback to perform redo, if redo is available.
    pub redo: Option<Callback<()>>,
    /// Callback to open the database-chooser.
    pub choose_db: Callback<()>,
    /// Callback to toggle displaying empty balances.
    pub toggle_empty: Callback<()>,
    /// Callback to toggle showing settings.
    pub open_settings: Callback<()>,
}

/// Displays the App header including titlebar and menubar.
#[function_component]
pub fn AppHeader(
    Props {
        dbname,
        hide_empty,
        choose_world,
        undo,
        redo,
        choose_db,
        toggle_empty,
        open_settings,
    }: &Props,
) -> Html {
    let left = html! {
        <>
            <Button title="Choose World" onclick={choose_world}>
                {material_icon("folder_open")}
            </Button>
            <Button title="Undo" onclick={undo}>
                {material_icon("undo")}
            </Button>
            <Button title="Redo" onclick={redo}>
                {material_icon("redo")}
            </Button>
            <Button title="Choose Database" onclick={choose_db}>
                {material_icon("factory")}
                <span>{dbname}</span>
            </Button>
            <Button class="hide-empty-button" title="Hide Empty Balances" onclick={toggle_empty}>
                {material_icon("exposure_zero")}
                if *hide_empty {
                    {material_icon("visibility_off")}
                } else {
                    {material_icon("visibility")}
                }
            </Button>
        </>
    };

    let right = html! {
        <>
            <Button title="Settings" onclick={open_settings}>
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
