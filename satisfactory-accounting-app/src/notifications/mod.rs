use yew::{function_component, html, use_callback, use_state_eq, Html};

use crate::bugreport::file_a_bug;
use crate::inputs::button::Button;
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

/// Versions of the notification message used in ack numbers..
mod versions {
    pub(super) const V1M2P9: u32 = 2;
    pub(super) const V1M2P10: u32 = 3;
    pub(super) const V1M2P11: u32 = 4;
    pub(super) const V1M2P13: u32 = 5;

    pub(super) const PREVIOUS: u32 = V1M2P11;
    pub(super) const CURRENT: u32 = V1M2P13;
}

struct Notification {
    title: &'static str,
    content: Html,
}

/// Displays notifications to the user.
#[function_component]
pub fn Notifications() -> Html {
    let user_settings = use_user_settings();
    let user_settings_dispatcher = use_user_settings_dispatcher();

    let notifications_displayed = use_state_eq(|| true);
    let hide_notifications = use_callback(notifications_displayed.setter(), |(), setter| {
        setter.set(false);
    });

    let ack_notifications = use_callback(user_settings_dispatcher, |(), dispatcher| {
        dispatcher.ack_notification(versions::CURRENT);
    });

    html! {
        if *notifications_displayed {
            if let Some(notification) = get_notification(user_settings.acked_notification) {
                <OverlayWindow title={notification.title} class="Notifications">
                    {notification.content}
                    <div class="dismiss-buttons">
                        <Button title="Hide this message until next session" onclick={hide_notifications}>
                            {"Hide"}
                        </Button>
                        <Button title="Dismiss this notification" onclick={ack_notifications}>
                            {"Dismiss"}
                        </Button>
                    </div>
                </OverlayWindow>
            }
        }
    }
}

fn get_notification(acked_version: u32) -> Option<Notification> {
    if acked_version == 0 {
        Some(get_new_user_welcome())
    } else if acked_version < versions::CURRENT {
        Some(get_existing_user_notification(acked_version))
    } else {
        None
    }
}

fn get_new_user_welcome() -> Notification {
    Notification {
        title: "Welcome to Satisfactory Accounting!",
        content: html! {
            <>
                <h2>{"Greetings, Pioneer."}</h2>
                <p>{
                    "Satisfactory Accounting is a tool intended to help you keep track of what \
                    you've actually built in your factory (hence \"accounting\"), and to help you \
                    plan out new factories, if you prefer to manually enter things. It's sort of a \
                    very fancy spreadsheet which already knows about the various buildings, items, \
                    and recipes in the game, so you can focus on what you factories produce, \
                    rather than trying to make a spreadsheet know how many rods you get in that \
                    one alternate recipe you just unlocked"
                }</p>
                <p>{
                    "In the tool, you can add Buildings and Groups and can move them around with \
                    drag and drop. You can select buildings from the game and choose which recipes \
                    they should be using, and the tool will calcuate the inputs and outputs for \
                    that building based on the recipe/item, purity, clock speed, and a multiplier \
                    which lets you say how many copies of a building you have."
                }</p>
                <p>{
                    "The inputs and outpus of a building are then added to the net intput/output \
                    of whatever group that building belongs to, and so on up to the top of the \
                    world, so you can see how many of each resource you produce per minute both \
                    across a whole word and locally, in whatever level of groups you find useful."
                }</p>
                <p>{
                    "Unlike some other tools, Satisfactory Accounting doesn't calculate entire \
                    factory chains for you or pick out buildigns and recipies to use to get \
                    particular outputs. However it does support calculating how many copies of a \
                    buildng you need and at what clock speeds in order to produce or consume items \
                    at a particular rate, once you have chosen a building and recipe. If your \
                    factory planning style is similar to mine, you may find this more useful than \
                    a tool that just tells you a whole chain."
                }</p>
                <p>{
                    "My hope is that this tool will fill a niche in the Satisfactory community for \
                    people who want to keep track of what they build, but like me, don't want to \
                    have to manage a spreadsheet by hand."
                }</p>
                <h3>{"Enjoy!"}</h3>
                {signature()}
            </>
        },
    }
}

fn get_existing_user_notification(acked_version: u32) -> Notification {
    Notification {
        title: "Satisfactory Accounting v1.2.13",
        content: html! {
            <>
                <h2>{"Welcome back, Pioneer."}</h2>
                <p>{"This is a minor update which adds support for adjusting how values are \
                rounded in the app."}</p>
                <h3>{"What's in this version"}</h3>
                <ul>
                    <li>
                        <p><b>{"Configurable rounding modes."}</b>{" You can configure rounding \
                        for clock speeds, building multipliers, and balances. Additionally, you \
                        can choose whether the coloring in balances and 'hide empty balances' use \
                        the exact value or the rounded value."}</p>
                    </li>
                    <li>
                        <p><b>{"Increment/Decrement."}</b>{" When you select a building's \
                        multiplier, clock speed, or balance value, you can use the up and down \
                        arrow keys to increment and decrement it rather than typing a whole new \
                        number. These can also be combined with 'Shift' for finer adjustments."}</p>
                    </li>
                </ul>
                <h3>{"What's coming next"}</h3>
                <p>{"I believe this covers all the feature requests that I can reasonably cover \
                without substantial changes to how the app stores and tracks balances, so aside \
                from bug fixes, this is probably going to be the last update for a while. I do \
                have plans for more substantial updates, but I also have fairly limited free time \
                and I'm not sure if or when I'll prioritize them."}</p>
                if acked_version < versions::PREVIOUS {
                    <h3>{"Additionally, you may have missed these updates from previous releases:"}</h3>
                    if acked_version < versions::V1M2P11 {
                        <h4>{"Version 1.2.11"}</h4>
                        <ul>
                            <li>
                                <p><b>{"Fractional Multipliers"}</b>{" If a building supports \
                                overclocking, then it can now have a non-integer multiplier. When a \
                                building has a fractional multiplier, it represents N buildings at the \
                                current clock speed, plus one building at a reduced clock speed."}</p>
                                <p>{"For example, if you have a multiplier of 3.5, on a building with \
                                clock speed 1.0, then that means 3 buildings at 1.0 plus 1 building with \
                                clock speed 0.5.. If you have a multiplier of 3.5 and a clock speed of \
                                2.0, then that means 3 buildings with clock speed 2.0, plus 1 building with
                                clock speed 1.0."}</p>
                                <p>{"Note that clock speeds are clamped at the limits imposed by the game, \
                                so for example if you have a building at 0.1 clock speed and try to \
                                multiply that by 3.05, you'll end up with 3 @ 0.1 + 1 @ 0.01, not 3 @ 0.1 \
                                + 1 @ 0.005, which matches what you can do in game. Unless you frequently \
                                use very low clock speeds, this probably doesn't matter to you"}</p>
                            </li>
                            <li>
                                <p><b>{"Backdriving!"}</b>{" or \"let me type in the number of items\". \
                                You can now click and edit the number of items on a building directly, and \
                                the tool will calculate the number of buildings and clock speed you need \
                                to produce that number of items, and update the building to match the \
                                desired output rate. Since it doesn't change recipies, you can always type \
                                positive numbers, and it will know whether it's an input or an output."}</p>
                                <p>{"There are a couple different modes available. The tool can either \
                                produce buildings with a uniform clock speed set on all buildings, or it \
                                can set to have most buildings with the same clock speed plus only one \
                                with a different clock speed, using the fractional multipliers above. The \
                                latter mode is more useful if you want most of your buildings to have a \
                                clock speed of 1.0."}</p>
                                <p>{"You can find more about the two modes in the settings menu, and \
                                switch between them for different categories of buildings."}</p>
                            </li>
                        </ul>
                    }
                    if acked_version < versions::V1M2P10 {
                        <h4>{"Version 1.2.10"}</h4>
                        <ul>
                            <li>
                                <p><b>{"Upload-replace."}</b>{" Save files downloaded from the App now \
                                include a unique ID which identifies which world they are. When you upload \
                                a world file, the App now checks if the ID matches an existing world, and \
                                if it does, it will now give you an option to replace the existing world \
                                or upload the file as a new world. To avoid confusion, I've now made world \
                                IDs visible in the world list."}</p>
                                <p>{"Older world files from before this change won't contain unique IDs, \
                                so if you upload an older file, it will always upload as a new world. But \
                                all files you download after this should have IDs. If you know what you're
                                doing, you can also add the world ID to existing files, or change world \
                                IDs in the JSON files to control what world a file will upload as. That's \
                                not an option in the UI because I thought it would be simpler if \
                                upload-replace was automatic in the common case."}</p>
                                <p>{"If you've already shared world files a bunch, you may have multiple \
                                copies of a world with diverging IDs. To get them to match, you'll just \
                                have to pick one version to upload everywhere so every computer/person \
                                sharing the file has a version with the same ID, and then after that you \
                                should all get the option to upload-and-replace."}</p>
                            </li>
                            <li>
                                <p><b>{"World List Sorting."}</b>{" Until now, the world list was always \
                                sorted by world ID. Since world IDs are random, that means the order of \
                                worlds in the list was pretty random. Now the list is sorted by name by \
                                default and you can click the headings to change which column it sorts by."}
                                </p>
                            </li>
                        </ul>
                    }
                    if acked_version < versions::V1M2P9 {
                        <h4>{"Version 1.2.9"}</h4>
                        <ul>
                            <li><p><b>{"Download and Upload."}</b>
                                {" You can now download your worlds as a JSON file from the World Manager, \
                                and upload saved JSON files as new worlds. This lets you save worlds for \
                                backup purposes or transfer them to a different computer, or share with a \
                                friend."}</p>
                                <p>{"Quick note for those of you who figured out how to copy out the world \
                                JSON before this update: I've added a 'model_version' tag to the \
                                downloaded JSON file format so that I can ensure that future versions of \
                                Satisfactory Accounting remain compatible with older save files, even if I \
                                make changes to the world format. If you created world JSON files without \
                                using the download button, you'll probably need to add the 'model_version' \
                                tag to them. The current version tag is \"v1.2.*\"."}</p>
                                <p>{"For everyone else, the download button adds this tag itself \
                                so you don't have to worry about this!"}</p>
                            </li>
                            <li>
                                <b>{"\"Latest\" mode for factory versions."}</b>
                                {" With this addition, you no longer need to manually change the factory \
                                database version every time I fix a missing or incorrect recipe. You still \
                                have the option to pin you world at a particular version if you prefer not \
                                to have things change on you."}
                            </li>
                            <li>
                                <p><b>{"Grid alignment."}</b>
                                {" A whole bunch of things have now been laid out in a grid format instead \
                                of just flexible layouts they were previously. In particular, you'll \
                                notice that all buildings and sub-groups in a group have their clock \
                                speeds and most of their balances aligned now. Plus when you have balances \
                                sorted by inputs vs outputs (which is now the default sorting mode), all \
                                inputs and outputs at the same group level will be aligned to a grid, \
                                which I think makes it easer to read."}</p>
                                <p>{"One downside of this is that it  makes the app take up more width, so \
                                you may find you need to scroll horizontally more often. Sorry about that. \
                                I hope you find the improved organization more helpful than the extra \
                                width is inconvenient; let me know if not."}</p>
                            </li>
                            <li>
                                <b>{"Group collapse button."}</b>
                                {" The group collapse button is now on the left. This means that groups and
                                buildings now have the same number of buttons on the right, so their
                                multipliers and balances all line up neatly."}
                            </li>
                            <li>
                                <b>{"Storage Persistence."}</b>
                                {" I hadn't realized this before, but apparently browser local-storage can \
                                just get randomly deleted by the browser unless you request that it be \
                                persisted. Fortunately that is rare, and I hope none of you lost you \
                                factory sheets to this mistake, but fortunately now you have the option to \
                                enabled proper persisted storage to make sure that can't happen."}
                            </li>
                            <li>
                                <b>{"Notifications."}</b>
                                {" I didn't used to have a way to let you know when things changed, other \
                                than putting that little \"update available\" tag in the database version \
                                selector. Now I do! Hi!"}
                            </li>
                        </ul>
                    }
                }
                <h3>{"In case of issues"}</h3>
                <p>{"If you run into any problems with this release, you can "}{file_a_bug()}{". \
                (there's also a link in the top-right corner of the app, with the bug icon)."}</p>
                <p>{"The previous two versions of Satisfactory Accounting are also available, \
                should you need to switch back to them to work around bugs. "}<b>{"However"}</b>
                {", the older versions are not compatible with fractional building multipliers. If \
                any buildings in a world have non-integer multipliers, you won't be able to load \
                that world in the older versions."}</p>
                <p>{"You can find the older versions at these links:"}</p>
                <ul>
                    <li>
                        <a target="_blank" href="https://satisfactory-accounting.github.io/v1.2.11/">
                            {"https://satisfactory-accounting.github.io/v1.2.11/"}
                        </a>{"."}
                    </li>
                    <li>
                        <a target="_blank" href="https://satisfactory-accounting.github.io/v1.2.10/">
                            {"https://satisfactory-accounting.github.io/v1.2.10/"}
                        </a>{"."}
                    </li>
                    <li>
                        <a target="_blank" href="https://satisfactory-accounting.github.io/v1.2.9/">
                            {"https://satisfactory-accounting.github.io/v1.2.9/"}
                        </a>{"."}
                    </li>
                    <li>
                        <a target="_blank" href="https://satisfactory-accounting.github.io/v1.2.8/">
                            {"https://satisfactory-accounting.github.io/v1.2.8/"}
                        </a>{"."}
                    </li>
                </ul>
                <h3><a target="_blank" href="https://youtu.be/79DijItQXMM">{"You're welcome!"}</a></h3>
                {signature()}
            </>
        },
    }
}

/// Gets my signature.
fn signature() -> Html {
    html! {
        <h3 class="signature" title="From \u{2013} zstewart">
            {"\u{2014}"}
            <img class="sig-logo" src="/images/logos/signature.svg" />
        </h3>
    }
}
