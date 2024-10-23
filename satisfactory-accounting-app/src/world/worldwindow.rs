use std::cell::RefCell;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use yew::{
    classes, function_component, hook, html, use_callback, use_context, use_mut_ref, Html,
    Properties,
};

use crate::inputs::button::Button;
use crate::material::material_icon;
use crate::modal::{use_modal_dispatcher, CancelDelete, ModalHandle, ModalOk};
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
use crate::world::{
    use_save_file_fetcher, use_world_list, use_world_list_dispatcher, DatabaseVersionSelector,
    FetchSaveFileError, WorldId, WorldMetadata,
};

pub type WorldChooserWindowManager = WindowManager<WorldChooserWindow>;
pub type WorldChooserWindowDispatcher = ShowWindowDispatcher<WorldChooserWindow>;

/// Gets access to the DbChooser window dispatcher which controls showing the user settings window.
#[hook]
pub fn use_world_chooser_window() -> WorldChooserWindowDispatcher {
    use_context::<WorldChooserWindowDispatcher>().expect(
        "use_world_chooser_window can only be used from within a child of DbChooserWindowManager",
    )
}

/// Shows the database chooser window.
#[function_component]
pub fn WorldChooserWindow() -> Html {
    let window_dispatcher = use_world_chooser_window();
    let close = use_callback(window_dispatcher, |(), window_dispatcher| {
        window_dispatcher.hide_window();
    });

    let world_list = use_world_list();
    let world_list_dispatcher = use_world_list_dispatcher();

    let create_world = use_callback(world_list_dispatcher, |(), world_list_dispatcher| {
        world_list_dispatcher.create_world();
    });

    let world_rows = world_list.iter().map(|meta_ref| {
        html! {
            <WorldListRow id={meta_ref.id()} selected={meta_ref.is_selected()}
                meta={meta_ref.meta().clone()} />
        }
    });

    html! {
        <OverlayWindow title="Choose World" class="WorldChooserWindow" on_close={close}>
            <div class="overview">
                <p>{"Satisfactory Accounting allows you to have multiple worlds. You can create \
                new ones and switch between them here."}</p>
            </div>
            <div class="world-rows">
                <div class="create-button-row">
                    <span class="world-name">{"World Name"}</span>
                    <span class="world-version">{"World Version"}</span>
                    <span class="create-upload">
                        <Button class="green" title="Upload">
                            {material_icon("upload")}
                            <span>{"Upload World"}</span>
                        </Button>
                        <Button class="green" onclick={create_world} title="Create">
                            {material_icon("add")}
                            <span>{"Create New World"}</span>
                        </Button>
                    </span>
                </div>
                {for world_rows}
            </div>
        </OverlayWindow>
    }
}

#[derive(PartialEq, Properties)]
struct WorldListRowProps {
    /// ID of this world.
    id: WorldId,
    /// Whether this world was selected.
    selected: bool,
    /// Metadata for this world.
    meta: WorldMetadata,
}

/// Shows a single row in the DbChooserWindow.
#[function_component]
fn WorldListRow(
    &WorldListRowProps {
        id,
        selected,
        ref meta,
    }: &WorldListRowProps,
) -> Html {
    let dispatcher = use_world_list_dispatcher();
    let select_world = use_callback((id, dispatcher.clone()), |(), (id, dispatcher)| {
        dispatcher.set_world(*id);
    });

    let modal_handle: Rc<RefCell<Option<ModalHandle>>> = use_mut_ref(Default::default);
    let modals = use_modal_dispatcher();

    let save_file_fetcher = use_save_file_fetcher();
    let download = use_callback(
        (id, meta.name.clone(), modals.clone(), save_file_fetcher),
        |(), (id, name, modals, fetcher)| {
            let save_file = match fetcher.get_save_file(*id) {
                Ok(save_file) => save_file,
                Err(FetchSaveFileError::StorageError(StorageError::KeyNotFound(_))) => {
                    return modals
                        .builder()
                        .class("world-download-error")
                        .kind(ModalOk::close())
                        .title("World content not found")
                        .content(html! {
                            <>
                                <p>{"The content for world "}{name}{" was not found in your \
                                browser, so we are unable to download it. Sorry about that."}</p>
                            </>
                        })
                        .build()
                        .persist();
                }
                _ => todo!(),
            };
        },
    );

    let delete_forever = use_callback((id, dispatcher), |(), (id, dispatcher)| {
        dispatcher.delete_world(*id);
    });

    let delete_world = use_callback(
        (modals, delete_forever, meta.name.clone()),
        move |(), (modals, delete_forever, name)| {
            let modal = modals
                .builder()
                .title("Confirm Delete")
                .content(html! {
                   <div class="delete-content">
                       <p>{"Are you sure you want to delete the world "}{name}{"?"}</p>
                       <h2>{"This CANNOT be undone!"}</h2>
                   </div>
                })
                .class("modal-delete-forever")
                .kind(CancelDelete::delete(delete_forever.clone()))
                .build();
            *modal_handle.borrow_mut() = Some(modal);
        },
    );

    let classes = classes!("WorldListRow", selected.then_some("selected"));

    html! {
        <div class={classes}>
            <span class="world-name">{&meta.name}</span>
            <span class="world-version">
                {meta.database.map(DatabaseVersionSelector::name)}
            </span>
            if !selected {
                <Button key="switch" class="green switch-to-world" title="Switch to this World" onclick={select_world}>
                    if meta.load_error {
                        {material_icon("warning")}
                    } else {
                        {material_icon("open_in_browser")}
                    }
                </Button>
            }
            <Button key="download" class="download-world" title="Download World">
                if meta.load_error {
                    {material_icon("warning")}
                } else {
                    {material_icon("download")}
                }
            </Button>
            <Button key="delete" class="red delete-world" title="Delete World" onclick={delete_world}>
                {material_icon("delete")}
            </Button>
        </div>
    }
}
