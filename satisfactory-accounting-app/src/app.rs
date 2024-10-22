use yew::{function_component, html, Html};

// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use crate::appheader::AppHeader;
use crate::modal::ModalManager;
use crate::node_display::NodeTreeDisplay;
use crate::storagenotice::StorageNotice;
use crate::user_settings::{UserSettingsManager, UserSettingsWindowManager};
use crate::world::{DbChooserWindowManager, WorldChooserWindowManager, WorldManager};

#[function_component]
pub fn App() -> Html {
    html! {
        <ModalManager>
        <UserSettingsManager>
        <WorldManager>
            <div class="App">
                <UserSettingsWindowManager>
                <WorldChooserWindowManager>
                <DbChooserWindowManager>
                    <AppHeader />
                </DbChooserWindowManager>
                </WorldChooserWindowManager>
                </UserSettingsWindowManager>
                <NodeTreeDisplay />
            </div>
        </WorldManager>
        <StorageNotice />
        </UserSettingsManager>
        </ModalManager>
    }
}
