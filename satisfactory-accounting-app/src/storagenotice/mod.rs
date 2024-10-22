use std::rc::Rc;

use yew::{function_component, html, use_callback, use_reducer_eq, Html, Reducible};

use crate::inputs::button::Button;
use crate::material::material_icon;
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

const CURRENT_LOCAL_STORAGE_NOTICE_VERSION: u32 = 1;

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
struct Toggle(bool);

impl Reducible for Toggle {
    type Action = ();

    fn reduce(mut self: Rc<Self>, _: Self::Action) -> Rc<Self> {
        let this = Rc::make_mut(&mut self);
        this.0 = !this.0;
        self
    }
}

#[function_component]
pub fn StorageNotice() -> Html {
    let user_settings = use_user_settings();
    let user_settings_dispatcher = use_user_settings_dispatcher();

    let expanded = use_reducer_eq(Toggle::default);
    let toggle_expanded = use_callback(expanded.dispatcher(), |(), dispatcher| {
        dispatcher.dispatch(())
    });

    let accept = use_callback(user_settings_dispatcher.clone(), |(), dispatcher| {
        dispatcher.ack_local_storage(CURRENT_LOCAL_STORAGE_NOTICE_VERSION);
    });
    let accept_and_persist = use_callback(user_settings_dispatcher, |(), dispatcher| {
        dispatcher.ack_local_storage(CURRENT_LOCAL_STORAGE_NOTICE_VERSION);
        dispatcher.persist_local_storage();
    });

    html! {
        if user_settings.acked_local_storage_notice_version < CURRENT_LOCAL_STORAGE_NOTICE_VERSION {
            <OverlayWindow class="StorageNotice">
                <div class="local-storage-ack-headline">
                    <div>
                        <p>
                            <b>{"Local Storage Notice:"}</b>
                            {" Satisfactory Accounting uses "}
                            <a target="_blank" href="https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API">
                                {"Local Storage"}
                            </a>
                            {", a technology similar to cookies, to store your user settings and \
                            satisfactory worlds. This means that your worlds are stored on your own \
                            computer or device."}
                        </p>
                    </div>
                    <Button title="More" onclick={toggle_expanded}>
                        if expanded.0 {
                            {material_icon("expand_more")}
                            <span>{"Less"}</span>
                        } else {
                            {material_icon("expand_less")}
                            <span>{"More"}</span>
                        }
                    </Button>
                </div>
                if expanded.0 {
                    <div>
                        <h1>{"Usage"}</h1>
                        <p>{"Satisfactory Accounting uses local storage for the following purposes:"}</p>
                        <ul>
                            <li>{"To store your user preferences, including balance sort and \
                                display modes, as well as acknowledgement of various welcome \
                                messages and notices"}</li>
                            <li>{"To store your worlds"}</li>
                        </ul>
                        <p>{"That's it. Satisfactory Accounting does not currently have any \
                            server-side data storage, and does not track you in any way (I \
                            literally have no idea how many of you use this tool!)"}</p>
                        <p>{"All of our local storage usage is strictly necessary to provide you \
                            with the ability to track factory production, so there are no optional \
                            cookies for you to disable at this time."}</p>
                        <h2>{"Deletion"}</h2>
                        <p>{"Since all data is stored on your machine, you can delete all data \
                            from Satisfactory Accounting by clearing your browser's history/cache."}</p>
                        <h2>{"Best Effort Storage"}</h2>
                        <p>{"By default, web browsers provide Local Storage on a best-effort \
                            basis. What this means is that your browser can choose to delete \
                            items from Local Storage if storage capacity is low. In practice, \
                            this is unlikely, especially for websites you visit regularly, \
                            however there is a chance that your browser could delete your \
                            factories from this site."}</p>
                        <p>{"You can change this storage from Best Effort to Persisted, in \
                            which case your browser should only delete it if you ask it to by \
                            choosing \"Accept and Persist\" below. This is a separate setting \
                            as it may require you to grant permission in your browser. You can \
                            revoke this permission at any time from your browser's permissions \
                            manager."}</p>
                        <p>{"If you don't persist local storage now, you can always opt in \
                            later from the settings menu."}</p>
                        <p>{"We don't know if you have enabled persistence already since browsers \
                            often do not report on local storage persistence accurately, so you \
                            may see the option to enable persistence even if you already have \
                            persistence enabled. Note that we also do not have the ability to \
                            disable persistence; the only way to do that is from your browser's \
                            permissions manager. That means that if you have already enabled \
                            persistence, choosing \"Accept with Best Effort storage\" will "}
                            <b>{"not"}</b>{" change you from \"Persisted\" to \"Best Effort\"."}</p>
                        <div class="accept-buttons">
                            <Button onclick={accept}>
                                {"Accept with Best Effort storage"}
                            </Button>
                            <Button onclick={accept_and_persist}>
                                {"Accept and Persist storage"}
                            </Button>
                        </div>
                    </div>
                }
            </OverlayWindow>
        }
    }
}
