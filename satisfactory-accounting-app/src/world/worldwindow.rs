use std::cell::RefCell;
use std::rc::Rc;

use gloo::file::{Blob, ObjectUrl};
use gloo::storage::errors::StorageError;
use log::error;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;
use yew::{
    classes, function_component, hook, html, use_callback, use_context, use_mut_ref, AttrValue,
    Callback, Html, Properties,
};

use crate::bugreport::file_a_bug;
use crate::inputs::button::{Button, UploadButton, UploadedFile};
use crate::material::material_icon;
use crate::modal::{use_modal_dispatcher, CancelDelete, ModalDispatcher, ModalHandle, ModalOk};
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

    let upload_world = use_callback(
        world_list_dispatcher.clone(),
        |file: UploadedFile, world_list_dispatcher| {
            world_list_dispatcher.upload_world(file.name, file.data);
        },
    );

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
                        <UploadButton class="green" title="Upload" onupload={upload_world}>
                            {material_icon("upload")}
                            <span>{"Upload World"}</span>
                        </UploadButton>
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

    let delete_forever = use_callback((id, dispatcher), |(), (id, dispatcher)| {
        dispatcher.delete_world(*id);
    });

    let download = use_download_callback(id, meta.name.clone(), modals.clone());

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
            <Button key="download" class="download-world" title="Download World" onclick={download}>
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

#[hook]
fn use_download_callback(id: WorldId, name: AttrValue, modals: ModalDispatcher) -> Callback<()> {
    // This just keeps the download url alive as long as the world list row isn't disposed, and
    // ensures it gets cleaned up when the world chooser is closed.
    let download_url_retainer: Rc<RefCell<Option<ObjectUrl>>> = use_mut_ref(|| None);
    let save_file_fetcher = use_save_file_fetcher();

    use_callback(
        (id, name, modals, save_file_fetcher),
        // We need move here to move download_url_retainer, as that is shared but not treated as a
        // dependency, since we only need it to exist to dump the object url into so it stays alive.
        move |(), (id, name, modals, fetcher)| {
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
                                <p>{"The content for world \""}{name}{"\" was not found in your \
                                browser's storage, so we are unable to download it. Sorry about
                                that."}</p>
                            </>
                        })
                        .build()
                        .persist();
                }
                Err(FetchSaveFileError::StorageError(e)) => {
                    return modals
                        .builder()
                        .class("world-download-error")
                        .kind(ModalOk::close())
                        .title("World could not be loaded")
                        .content(html! {
                            <>
                                <p>{"We were unable to load the content for world \""}{name}{"\". \
                                Your world data seems to still be present, so this may be \
                                recoverable. For help you can "}{file_a_bug()}{". If you file a \
                                bug, please include this error message:"}</p>
                                <pre>
                                    {"Unable to load world "}{id}{": "}{e}
                                </pre>
                            </>
                        })
                        .build()
                        .persist();
                }
            };
            let json = match serde_json::to_string(&save_file) {
                Ok(json) => json,
                Err(e) => {
                    return modals
                        .builder()
                        .class("world-download-error")
                        .kind(ModalOk::close())
                        .title("World could not be serialized")
                        .content(html! {
                            <>
                                <p>{"We successfully loaded your world but couldn't serialize it \
                                to create the download file for some reason. This is probably a \
                                bug, and you can "}{file_a_bug()}{". If you file a bug, please \
                                include this error message:"}</p>
                                <pre>
                                    {"Unable serialize world: "}{e}
                                </pre>
                            </>
                        })
                        .build()
                        .persist();
                }
            };
            let blob = Blob::new_with_options(json.as_str(), Some("application/json"));
            let url = ObjectUrl::from(blob);

            // To trigger the download, we create an anchor tag that isn't attached to the document
            // and click it.
            let a = match gloo::utils::document().create_element("a") {
                Ok(a) => match a.dyn_into::<HtmlAnchorElement>() {
                    Ok(a) => a,
                    Err(elem) => {
                        error!("Unable to cast element {elem:?} to HtmlAnchorElement");
                        return;
                    }
                },
                Err(e) => {
                    error!("Unable to create an 'a' element to download with: {e:?}");
                    return;
                }
            };
            a.set_href(&url);
            let filename = if name.is_empty() {
                format!("SatisfactoryAccounting-{}.json", id.as_unprefixed())
            } else {
                format!("{name}-{}.json", id.as_unprefixed())
            };
            a.set_download(&filename);
            a.click();

            *download_url_retainer.borrow_mut() = Some(url.clone());
        },
    )
}
