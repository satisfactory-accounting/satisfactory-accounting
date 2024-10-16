use satisfactory_accounting::database::DatabaseVersion;
use yew::{classes, function_component, hook, html, use_callback, use_context, Html, Properties};

use crate::inputs::toggle::{MaterialCheckbox, MaterialRadio};
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};
use crate::world::{use_db_controller, DatabaseVersionSelector};

pub type DbChooserWindowManager = WindowManager<DbChooserWindow>;
pub type DbChooserWindowDispatcher = ShowWindowDispatcher<DbChooserWindow>;

/// Gets access to the DbChooser window dispatcher which controls showing the user settings window.
#[hook]
pub fn use_db_chooser_window() -> DbChooserWindowDispatcher {
    use_context::<DbChooserWindowDispatcher>().expect(
        "use_db_chooser_window can only be used from within a child of DbChooserWindowManager",
    )
}

/// Shows the database chooser window.
#[function_component]
pub fn DbChooserWindow() -> Html {
    let window_dispatcher = use_db_chooser_window();
    let close = use_callback(window_dispatcher, |(), window_dispatcher| {
        window_dispatcher.hide_window();
    });

    let user_settings = use_user_settings();
    let settings_dispatcher = use_user_settings_dispatcher();

    let toggle_show_deprecated = use_callback(settings_dispatcher, |_, settings_dispatcher| {
        settings_dispatcher.toggle_show_deprecated();
    });

    let databases = DatabaseVersion::ALL
        .iter()
        .rev()
        .filter(|version| user_settings.show_deprecated_databases || !version.is_deprecated())
        .map(|&version| {
            html! { <DbListRow version={DatabaseVersionSelector::Pinned(version)} /> }
        });

    html! {
        <OverlayWindow title="Choose Database" class="DbChooserWindow" on_close={close}>
            <div class="overview">
                <p>{"Select which version of the Satisfactory Accounting recipe database to use. \
                This affects which buildings and recipes are available. New versions are added \
                both for new versions of Satisfactory and for new versions of Satisfactory \
                Accounting, for example for bug fixes when we find recipes in the database which \
                don't match the game."}</p>
                <p>{"Each version has a description which should note roughly what was changed in \
                that version. The version names are arbitrary and just chosen to be unique to \
                particular versions."}</p>
                <p>{"If you choose a specific version, your world will be pinned at that version
                until you change it. If you choose \"Latest\", your world will be updated to the
                latest database version when it is loaded."}</p>
                <p>{"Note that version changes are non-destructive. If you change database version \
                and an item or recipe is missing in the new version, it won't display properly, \
                but you can always change back to the previous database version without losing \
                anything."}</p>
                <label class="show-deprecated">
                    <span>{"Show deprecated versions"}</span>
                    <MaterialCheckbox checked={user_settings.show_deprecated_databases}
                        onclick={toggle_show_deprecated} />
                </label>
            </div>
            <div class="versions">
                <DbListRow version={DatabaseVersionSelector::Latest} />
                {for databases}
            </div>
        </OverlayWindow>
    }
}

#[derive(Properties, PartialEq)]
struct DbListRowProps {
    /// Version to display.
    version: DatabaseVersionSelector,
}

/// Shows a single row in the DbChooserWindow.
#[function_component]
fn DbListRow(&DbListRowProps { version }: &DbListRowProps) -> Html {
    let db_controller = use_db_controller();

    let classes = classes!("DbListRow", version.is_deprecated().then_some("deprecated"));
    let checked = db_controller.current_selector() == Some(version);
    let onclick = use_callback(
        (version, db_controller.dispatcher()),
        |_, (version, dispatcher)| {
            dispatcher.set_database(*version);
        },
    );

    html! {
        <label class={classes}>
            <span class="version-name">{version.name()}</span>
            <span class="version-description">{version.description()}</span>
            <MaterialRadio name="db-choice" {checked} {onclick}/>
        </label>
    }
}
