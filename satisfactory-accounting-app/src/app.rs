use yew::{classes, function_component, html, Html};

// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use crate::appheader::AppHeader;
use crate::user_settings::{UserSettingsManager, UserSettingsWindowManager};
use crate::world::{DbChooserWindowManager, WorldManager};

#[function_component]
pub fn App() -> Html {
    //     let link = ctx.link();
    //     let replace = link.callback(|(idx, replacement)| {
    //         assert!(idx == 0, "Attempting to replace index {} at the root", idx);
    //         Msg::ReplaceRoot { replacement }
    //     });
    //     let set_metadata = link.callback(|(id, meta)| Msg::UpdateMetadata { id, meta });
    //     let batch_set_metadata = link.callback(|updates| Msg::BatchUpdateMetadata { updates });
    //     let move_node =
    //         Callback::from(|_| warn!("Root node tried to ask parent to move one of its children"));
    html! {
        <UserSettingsManager>
        <WorldManager>
            <div class="App">
                <UserSettingsWindowManager>
                <DbChooserWindowManager>
                    <AppHeader />
                </DbChooserWindowManager>
                </UserSettingsWindowManager>
                // TODO: hide empty balances.
                <div class={classes!("appbody")}>
                    //<NodeDisplay node={self.world.root.clone()}
                    //    path={Vec::new()}
                    //    {replace} {set_metadata} {batch_set_metadata}
                    //    {move_node} />
                </div>
            </div>
        </WorldManager>
        </UserSettingsManager>
    }
}

// impl App {
//     fn app_header(&self) -> Html {
//         let db_choice = &self.world.database;
//
//         let on_choose_world = &self.toggle_world_chooser;
//         let on_undo = (!self.undo_stack.is_empty()).then_some(self.undo.clone());
//         let on_redo = (!self.redo_stack.is_empty()).then_some(self.redo.clone());
//         let on_choose_db = &self.toggle_db_chooser;
//         html! {
//             <AppHeader {db_choice} {on_choose_world} {on_undo} {on_redo} {on_choose_db} />
//         }
//     }
//
//     fn world_chooser(&self, ctx: &Context<Self>) -> Html {
//         let link = ctx.link();
//         let close = link.callback(|_| Msg::ToggleWindow(OverlayWindow::None));
//         let new = link.callback(|_| Msg::CreateWorld);
//
//         let worlds = self.worlds.worlds.iter().map(|(&id, meta)| {
//             let load_error = meta.load_error;
//             let open = link.callback(move |_| Msg::SetWorld(id));
//             let delete = link.callback(move |_| Msg::InitiateDelete(id));
//             html! {
//                 <div class="world-list-row">
//                     <span>{&meta.name}</span>
//                     <span class="right-buttons">
//                         <button class="delete-world" title="Delete World" onclick={delete}>
//                             <span class="material-icons">{"delete"}</span>
//                         </button>
//                         if load_error {
//                             <span class="BuildError material-icons error" title="Unable to load this world in this version of Satisfactory Accounting">
//                                 {"warning"}
//                             </span>
//                         } else {
//                             <button class="new-world" title="Switch to this World" onclick={open}>
//                                 <span class="material-icons">{"open_in_browser"}</span>
//                             </button>
//                         }
//                     </span>
//                 </div>
//             }
//         });
//         let hidden = match self.overlay_window {
//             OverlayWindow::WorldChooser => None,
//             _ => Some("hide"),
//         };
//         html! {
//             <div class={classes!("overlay-window", hidden)}>
//                 <div class="close-bar">
//                     <h3>{"Choose World"}</h3>
//                     <button class="close" title="Close" onclick={close}>
//                         <span class="material-icons">{"close"}</span>
//                     </button>
//                 </div>
//                 <div class="world-list">
//                     <div class="world-list-row">
//                         <span>{"Create New"}</span>
//                         <button class="new-world" title="Create New World" onclick={new}>
//                             <span class="material-icons">{"add"}</span>
//                         </button>
//                     </div>
//                     { for worlds }
//                 </div>
//             </div>
//         }
//     }
//
//     fn confirm_delete(&self, ctx: &Context<Self>, id: WorldId) -> Html {
//         let link = ctx.link();
//         let cancel = link.callback(|_| Msg::CancelDelete);
//         let delete = link.callback(move |_| Msg::DeleteForever(id));
//
//         let name = match self.worlds.worlds.get(&id) {
//             Some(meta) if !meta.name.is_empty() => &meta.name,
//             Some(_) => "<Empty Name>",
//             None => "<Not Found>",
//         };
//         html! {
//             <div class="overlay-delete-window">
//                 <h2>{"Are you sure you want to delete World "}{name}</h2>
//                 <h3>{"This CANNOT be undone!"}</h3>
//                 <div class="button-row">
//                     <button class="cancel" title="Cancel" onclick={cancel}>
//                         <span>{"Cancel"}</span>
//                         <span class="material-icons">{"arrow_back"}</span>
//                     </button>
//                     <button class="delete-forever" title="Delete Forever" onclick={delete}>
//                         <span>{"Delete"}</span>
//                         <span class="material-icons">{"delete_forever"}</span>
//                     </button>
//                 </div>
//             </div>
//         }
//     }
// }
