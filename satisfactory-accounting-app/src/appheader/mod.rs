use std::borrow::Cow;

use yew::{function_component, html, use_callback, Html};

use menubar::MenuBar;
use titlebar::TitleBar;

use crate::inputs::button::{Button, LinkButton};
use crate::material::material_icon;
use crate::user_settings::{
    use_user_settings, use_user_settings_dispatcher, use_user_settings_window,
};
use crate::world::{
    use_db_chooser_window, use_db_controller, use_undo_controller, use_world_chooser_window,
    DatabaseVersionSelector,
};

mod menubar;
mod titlebar;

/// Displays the App header including titlebar and menubar.
#[function_component]
pub fn AppHeader() -> Html {
    let world_window_dispatcher = use_world_chooser_window();
    let on_choose_world = use_callback(world_window_dispatcher, |(), world_window_dispatcher| {
        world_window_dispatcher.toggle_window();
    });

    let undo_controller = use_undo_controller();
    let on_undo = use_callback(undo_controller.dispatcher(), |(), undo_dispatcher| {
        undo_dispatcher.undo();
    });
    let on_redo = use_callback(undo_controller.dispatcher(), |(), undo_dispatcher| {
        undo_dispatcher.redo();
    });

    let db_controller = use_db_controller();
    let db_window_dispatcher = use_db_chooser_window();
    let on_db = use_callback(db_window_dispatcher, |(), db_window_dispatcher| {
        db_window_dispatcher.toggle_window();
    });

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
            <Button title="Undo" onclick={on_undo} disabled={!undo_controller.has_undo()}>
                {material_icon("undo")}
            </Button>
            <Button title="Redo" onclick={on_redo} disabled={!undo_controller.has_redo()}>
                {material_icon("redo")}
            </Button>
            <Button title="Choose Database" onclick={on_db}>
                {material_icon("factory")}
                <span>{db_name(db_controller.current_selector())}</span>
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
fn db_name(version: Option<DatabaseVersionSelector>) -> Cow<'static, str> {
    match version {
        Some(version) => {
            if version.is_deprecated() {
                Cow::Owned(format!("{} \u{2013} Update Available!", version.name()))
            } else {
                Cow::Borrowed(version.name())
            }
        }
        None => Cow::Borrowed("Custom"),
    }
}
