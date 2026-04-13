// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use serde::{Deserialize, Serialize};
use yew::{classes, function_component, html, use_callback, Children, Classes, Html, Properties};

use crate::appheader::AppHeader;
use crate::inputs::toggle::MaterialRadio;
use crate::modal::ModalManager;
use crate::node_display::HighlightItemManager;
use crate::node_display::NodeTreeDisplay;
use crate::notifications::Notifications;
use crate::storagenotice::StorageNotice;
use crate::user_settings::{
    use_user_settings, use_user_settings_dispatcher, UserSettingsManager, UserSettingsWindowManager,
};
use crate::world::{DbChooserWindowManager, WorldChooserWindowManager, WorldManager};

#[function_component]
pub fn App() -> Html {
    html! {
        <ModalManager>
        <UserSettingsManager>
        <WorldManager>
        <HighlightItemManager>
            <AppInner>
                <UserSettingsWindowManager>
                <WorldChooserWindowManager>
                <DbChooserWindowManager>
                    <AppHeader />
                </DbChooserWindowManager>
                </WorldChooserWindowManager>
                </UserSettingsWindowManager>
                <NodeTreeDisplay />
            </AppInner>
        </HighlightItemManager>
        </WorldManager>
        <Notifications />
        <StorageNotice />
        </UserSettingsManager>
        </ModalManager>
    }
}

#[derive(Properties, PartialEq)]
struct AppInnerProps {
    children: Children,
}

/// Inner implementation of App. Needed for app to access user settings to apply global display
/// classes.
#[function_component]
fn AppInner(props: &AppInnerProps) -> Html {
    let user_settings = use_user_settings();
    let classes = classes!("App", user_settings.global_display.classes());
    html! {
        <div class={classes}>
            {props.children.clone()}
        </div>
    }
}

/// Visual settings that apply to the whole app.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalDisplaySettings {
    /// Display mode to use for the App.
    size: SizeMode,
}

impl GlobalDisplaySettings {
    fn classes(&self) -> Classes {
        let size = match self.size {
            SizeMode::Default => None,
            SizeMode::Compact => Some("compact-mode"),
        };
        classes!(size)
    }

    pub fn update(&mut self, msg: GlobalDisplaySettingsMsg) -> bool {
        match msg.msg {
            GlobalDisplaySettingsAction::SetSizeMode(mode) => {
                if mode == self.size {
                    false
                } else {
                    self.size = mode;
                    true
                }
            }
        }
    }
}

/// How the app should be sized.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
enum SizeMode {
    /// Default size of the app.
    #[default]
    Default,
    /// Compact mode tries to make more elements fit on a single screen.
    Compact,
}

/// Container for commands that are used to update GlobalDisplaySettings.
#[repr(transparent)]
pub struct GlobalDisplaySettingsMsg {
    /// Inner message (non-pub).
    msg: GlobalDisplaySettingsAction,
}

impl From<GlobalDisplaySettingsAction> for GlobalDisplaySettingsMsg {
    #[inline]
    fn from(msg: GlobalDisplaySettingsAction) -> Self {
        Self { msg }
    }
}

enum GlobalDisplaySettingsAction {
    /// Sets the global size mode.
    SetSizeMode(SizeMode),
}

/// Displays the settings section for controlling backdrive settings.
#[function_component]
pub fn GlobalDisplaySettingsSection() -> Html {
    let user_settings = use_user_settings();
    let settings = &user_settings.global_display;
    let user_settings_dispatcher = use_user_settings_dispatcher();
    let set_default = use_callback(
        user_settings_dispatcher.clone(),
        |_, user_settings_dispatcher| {
            user_settings_dispatcher.update_global_display_settings(
                GlobalDisplaySettingsAction::SetSizeMode(SizeMode::Default),
            );
        },
    );
    let set_compact = use_callback(
        user_settings_dispatcher.clone(),
        |_, user_settings_dispatcher| {
            user_settings_dispatcher.update_global_display_settings(
                GlobalDisplaySettingsAction::SetSizeMode(SizeMode::Compact),
            );
        },
    );
    html! {
        <div class="settings-section">
            <h2>{"Global Display Settings"}</h2>
            <p>{"Controls which affect the overall appearance of the app."}</p>
            <div class="settings-subsection">
                <h3>{"Display Size"}</h3>
                <p>{"Affects the overall side of the app. Compact can fit larger more deeply \
                    nested worlds on one screen."}</p>
                <ul>
                    <li>
                        <label>
                            <span>{"Default"}</span>
                            <MaterialRadio
                                checked={settings.size == SizeMode::Default}
                                onclick={set_default} />
                        </label>
                    </li>
                    <li>
                        <label>
                            <span>{"Compact"}</span>
                            <MaterialRadio
                                checked={settings.size == SizeMode::Compact}
                                onclick={set_compact} />
                        </label>
                    </li>
                </ul>
            </div>
        </div>
    }
}
