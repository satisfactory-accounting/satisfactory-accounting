use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use gloo::file::{Blob, ObjectUrl};
use gloo::storage::errors::StorageError;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;
use yew::{
    classes, function_component, hook, html, use_callback, use_context, use_mut_ref, AttrValue,
    Callback, Html, Properties,
};

use crate::bugreport::file_a_bug;
use crate::inputs::button::{Button, UploadButton, UploadedFile};
use crate::material::material_icon;
use crate::modal::{
    use_modal_dispatcher, BinaryChoice, CancelDelete, ModalDispatcher, ModalHandle, ModalOk,
};
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};
use crate::world::manager::PendingUpload;
use crate::world::{
    use_save_file_fetcher, use_world_list, use_world_list_dispatcher, DatabaseVersionSelector,
    FetchSaveFileError, WorldId, WorldMetadata,
};

/// Message to control WorlSortSettings.
pub enum WorldSortSettingsMsg {
    /// Switch to this column if not selected, or toggle the sort direction of that column if
    /// selected.
    #[allow(private_interfaces)] // Can intentionally only be created from this module.
    ToggleColumn { column: SortColumn },
}

/// Sorting settings to apply to the world list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSortSettings {
    /// Which column to sort the world list by.
    column: SortColumn,
    /// Direction to sort the world list.
    direction: SortDirection,
}

impl WorldSortSettings {
    /// Toggles the specified column, either switching to it or swaping the sort direction of it.
    fn toggle_column(&mut self, column: SortColumn) -> bool {
        if self.column == column {
            self.direction.invert();
        } else {
            self.column = column;
            self.direction = SortDirection::Ascending;
        }
        true
    }

    /// Applies a sort settings message.
    pub fn update(&mut self, msg: WorldSortSettingsMsg) -> bool {
        match msg {
            WorldSortSettingsMsg::ToggleColumn { column } => self.toggle_column(column),
        }
    }
}

/// Sort order to use for worlds in the world window.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
enum SortColumn {
    /// Sort by the user world name (then by version, then by id).
    #[default]
    Name,
    /// Sort by the world version (then by name, then by id).
    Version,
    /// Sort by the world ID.
    WorldId,
}

/// Sort order to use for worlds in the world window.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
enum SortDirection {
    /// Sort in the normal natural order (a-z, 0-9).
    #[default]
    Ascending,
    /// Sort in reverse order (z-a, 9-0).
    Descending,
}

impl SortDirection {
    /// Inverts this sort order in place.
    fn invert(&mut self) {
        *self = match *self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        };
    }

    /// Apply this ordering to a comparison result. Assumes that Ascending matches the forward
    /// ordering and descending matches the revers ordering.
    fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            Self::Ascending => ordering,
            Self::Descending => ordering.reverse(),
        }
    }
}

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

    let modal_dispatcher = use_modal_dispatcher();
    // This is used to keep the modal alive until the world window is closed.
    let upload_modal_handle = use_mut_ref(|| None::<ModalHandle>);
    let on_matches_existing = use_callback(
        modal_dispatcher,
        move |pending: PendingUpload, modal_dispatcher| {
            let lhs = html! { <span>{"Upload as new World"}</span> };
            let rhs = html! { <span>{"Replace existing World"}</span> };
            let title = "Upload or Replace?";
            let content = html! { <>
                <p>{"The world you uploaded, named \""}{pending.uploaded_name()}{"\", \
                appears to match the ID ("}{pending.id().as_base64()}{") of a world you already \
                have, named \""}{pending.existing_name()}{"\"."}</p>
                <p>{"Would you like to upload the world as a new world, or replace the existing \
                one? If you replace the existing world, its state from before the upload will be \
                placed in the undo history, so you can undo this action."}</p>
            </> };
            let pending = Rc::new(RefCell::new(Some(pending)));
            let on_lhs = {
                let pending = pending.clone();
                Callback::from(move |()| {
                    if let Some(pending) = pending.take() {
                        pending.finish_as_new();
                    } else {
                        warn!("Pending upload already finished");
                    }
                })
            };
            let on_rhs = {
                Callback::from(move |()| {
                    if let Some(pending) = pending.take() {
                        pending.finish_replacing_existing();
                    } else {
                        warn!("Pending upload already finished");
                    }
                })
            };

            let handle = modal_dispatcher
                .builder()
                .title(title)
                .content(content)
                .class("upload-world-replace-choice")
                .kind(
                    BinaryChoice::new(lhs, rhs)
                        .lhs_title("Upload the world a new world with a new ID")
                        .rhs_title("Upload the world over the existing world the the same ID")
                        .on_lhs(on_lhs)
                        .on_rhs(on_rhs),
                )
                .build();
            *upload_modal_handle.borrow_mut() = Some(handle);
        },
    );

    let upload_world = use_callback(
        (world_list_dispatcher.clone(), on_matches_existing),
        |file: UploadedFile, (world_list_dispatcher, on_matches_existing)| {
            world_list_dispatcher.upload_world(file.name, file.data, on_matches_existing.clone());
        },
    );

    let create_world = use_callback(world_list_dispatcher, |(), world_list_dispatcher| {
        world_list_dispatcher.create_world();
    });

    let user_settings = use_user_settings();
    let user_settings_dispatcher = use_user_settings_dispatcher();

    let toggle_sort_name = use_callback(
        user_settings_dispatcher.clone(),
        |_, user_settings_dispatcher| {
            user_settings_dispatcher.update_world_sort_settings(
                WorldSortSettingsMsg::ToggleColumn {
                    column: SortColumn::Name,
                },
            )
        },
    );
    let toggle_sort_version = use_callback(
        user_settings_dispatcher.clone(),
        |_, user_settings_dispatcher| {
            user_settings_dispatcher.update_world_sort_settings(
                WorldSortSettingsMsg::ToggleColumn {
                    column: SortColumn::Version,
                },
            )
        },
    );
    let toggle_sort_id = use_callback(user_settings_dispatcher, |_, user_settings_dispatcher| {
        user_settings_dispatcher.update_world_sort_settings(WorldSortSettingsMsg::ToggleColumn {
            column: SortColumn::WorldId,
        })
    });

    let sort_direction = user_settings.world_sort_settings.direction;
    let mut sorted_world_list = world_list.iter().collect::<Vec<_>>();
    let collator = crate::locale::get_collator();
    match user_settings.world_sort_settings.column {
        SortColumn::Name => sorted_world_list.sort_by(|lhs, rhs| {
            // When sorting by name, we sort by version descending so later versions show on top.
            sort_direction.apply(
                collator
                    .compare(&lhs.name, &rhs.name)
                    .then_with(|| lhs.database.cmp(&rhs.database).reverse())
                    .then_with(|| lhs.id().cmp(&rhs.id())),
            )
        }),
        SortColumn::Version => sorted_world_list.sort_by(|lhs, rhs| {
            sort_direction.apply(
                lhs.database
                    .cmp(&rhs.database)
                    .then_with(|| collator.compare(&lhs.name, &rhs.name))
                    .then_with(|| lhs.id().cmp(&rhs.id())),
            )
        }),
        SortColumn::WorldId => {
            // sorted_world_list is already sorted by world_id since world_list is a BTreeMap of
            // world_ids, and we can never have duplicate IDs so we never need to do a sub-sort by
            // name.
            if sort_direction == SortDirection::Descending {
                sorted_world_list.reverse();
            }
        }
    }

    let world_rows = sorted_world_list.into_iter().map(|meta_ref| {
        html! {
            <WorldListRow id={meta_ref.id()} selected={meta_ref.is_selected()}
                meta={meta_ref.meta().clone()} />
        }
    });

    let sort_dir = match user_settings.world_sort_settings.direction {
        SortDirection::Ascending => "\u{25B4}",
        SortDirection::Descending => "\u{25BE}",
    };

    html! {
        <OverlayWindow title="Choose World" class="WorldChooserWindow" on_close={close}>
            <div class="overview">
                <p>{"Satisfactory Accounting allows you to have multiple worlds. You can create \
                new ones and switch between them here."}</p>
            </div>
            <div class="world-rows">
                <div class="create-button-row">
                    <a href="javascript:void(0)" onclick={toggle_sort_name} class="world-name">
                        if user_settings.world_sort_settings.column == SortColumn::Name {
                            {sort_dir}
                        }
                        <span>{"World Name"}</span>
                    </a>
                    <a href="javascript:void(0)" onclick={toggle_sort_version} class="world-version">
                        if user_settings.world_sort_settings.column == SortColumn::Version {
                            {sort_dir}
                        }
                        <span>{"World Version"}</span>
                    </a>
                    <a href="javascript:void(0)" onclick={toggle_sort_id} class="world-id">
                        if user_settings.world_sort_settings.column == SortColumn::WorldId {
                            {sort_dir}
                        }
                        <span>{"World Id"}</span>
                    </a>
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
            <span class="world-id">{id.as_base64().to_string()}</span>
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
                                    {format!("Unable to load world {id:?}: {e}")}
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
                format!("SatisfactoryAccounting-{}.json", id.as_base64())
            } else {
                format!("{name}-{}.json", id.as_base64())
            };
            a.set_download(&filename);
            a.click();

            *download_url_retainer.borrow_mut() = Some(url.clone());
        },
    )
}
