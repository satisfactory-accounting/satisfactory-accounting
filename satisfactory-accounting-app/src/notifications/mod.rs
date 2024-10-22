use yew::{function_component, html, use_callback, use_state_eq, Html};

use crate::inputs::button::Button;
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

/// Curent version to ack up to when the user dismisses notifications.
const CURRENT_NOTIFICATION_VERSION: u32 = 2;

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
        dispatcher.ack_notification(CURRENT_NOTIFICATION_VERSION);
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
    } else if acked_version < CURRENT_NOTIFICATION_VERSION {
        Some(get_existing_user_notification())
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
                    "Unlike some other tools, there's not a huge focus on factory planning, (this \
                    tool won't calculate production chanins for you, and it doesn't know anything \
                    about conveyor belt speeds), but you may also find it useful for planning if \
                    you, like me, prefer to just start picking some buildings and recipes and \
                    manually adjust the building counts and clock speeds until you get a ratio \
                    you're happy with."
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

fn get_existing_user_notification() -> Notification {
    Notification {
        title: "Satisfactory Accounting v1.2.9",
        content: html! {
            <>
                <h2>{"Welcome back, Pioneer."}</h2>
                <p>{
                    "You know how sometimes you decide you want to expand a factory, but the way \
                    it's set up makes that a pain, and you realize you really should have arranged \
                    everything "}<em>{"this way"}</em>{" instead of "}<em>{"that way"}</em>{"? \
                    And then you tear down half the factory to rebuild it from scratch but better?"
                }</p>
                <p>{
                    "There are a few features I've wanted to add to Satisfactory Accounting, but \
                    instead of just hanging those new features off the end on a floating \
                    foundation like we all sometimes do when we care more about getting that part \
                    than being pretty, I did the thing and rebuilt a substantial portion of the \
                    app first."
                }</p>
                <h3>{"What's in this version"}</h3>
                <p>{
                    "Although this version doesn't change all that much, there are a few changes \
                    you might notice."
                }</p>
                <ul>
                    <li>
                        <b>{"\"Latest\" mode for factory versions."}</b>
                        {" With this addition, you no longer need to manually change the factory \
                        database version every time I fix a missing or incorrect recipe. You still \
                        have the option to pin you world at a particular version if you prefer not \
                        to have things change on you."}
                    </li>
                    <li>
                        <b>{"Grid alignment."}</b>
                        {" A whole bunch of things have now been laid out in a grid format instead \
                        of just flexible layouts they were previously. In particular, you'll \
                        notice that all buildings and sub-groups in a group have their clock \
                        speeds and most of their balances aligned now. Plus when you have balances \
                        sorted by inputs vs outputs (which is now the default sorting mode), all \
                        inputs and outputs at the same group level will be aligned to a grid, \
                        which I think makes it easer to read. One downside of this is that it \
                        makes the app take up more width, so you may find you need to scroll \
                        horizontally more often. Sorry about that. I hope you find the improved \
                        organization more helpful than the extra width is inconvenient; let me \
                        know if not."}
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
                <h3>{"What might be coming (no promises)"}</h3>
                <p>{"Here are some things that I'm planning on doing in upcoming versions of \
                Satisfactory Accounting. I do have limited time to work on it, and I also would \
                like to actually, you know, "}<em>{"play the game"}</em>{" sometimes, so I can't \
                make any promises about when or if these things will actually come, but I'll let \
                you know when and if I get to them."}</p>
                <ul>
                    <li>
                        <b>{"Download Button."}</b>
                        {" I'm planning on adding a download button to the world manager, so you \
                        can back up you worlds, transfer them to other browsers, or share them \
                        with friends. And of course the corresponding upload button. This is \
                        pretty likely to come in the next couple weeks, as its the next thing I \
                        plan to work on."}
                    </li>
                    <li>
                        <b>{"Backdrive Mode."}</b>
                        {" This is "}
                        <a target="_blank" href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues/12">
                            {"the most requested feature"}
                        </a>
                        {" by GitHub issue \u{1f44d}. I'd like to implement it, though I might \
                        end up rewriting a bunch more stuff along the way. Or I'll do a floating \
                        foundation version first and fix it later, we'll see. The main issue is if \
                        the number of items you request isn't a multiple of the recipe \u{2013} \
                        you'd probably rather have 5 machines at 100% speed + 1 machine at 25% \
                        speed than 6 machines at 87.5% speed, but I don't have a great way to \
                        represent that currently, so we'll see."}
                    </li>
                    <li>
                        <b>{"Co-op Mode. (But probably not)"}</b>
                        {" I would kind of like to add a co-op mode/ability to share worlds online \
                        live, so you can collaborate with friends. But that's its own entire \
                        massive project, so like don't get your hopes up! It might be helpful to \
                        guage interest though \u{2013} if you want this, please "}
                        <a target="_blank" href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues/36">
                            {"add a \u{1f44d} on GitHub"}
                        </a>
                        {"if you have a GitHub account."}
                    </li>
                </ul>
                <h3>{"In case of issues"}</h3>
                <p>{"You know how you sometimes do one of those big rebuilds and then discover you \
                forgot to connect a conveyor or a powerline somewhere? That probably happened here \
                too, since I changed a lot, I probably messed something up. If you run into any \
                problems, you can "}
                <a target="_blank" href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues">
                    {"file an issue on my GitHub"}
                </a>
                {" (there's also a link in the top-right corner of the app, with the bug icon)."}</p>
                <p>{"The previous version of Satisfactory Accounting is also available, should you \
                need to switch back to it to work around bugs, at "}
                <a target="_blank" href="https://satisfactory-accounting.github.io/v1.2.8/">
                    {"https://satisfactory-accounting.github.io/v1.2.8/"}
                </a>{"."}
                </p>
                <h3>{"I hope you like it"}</h3>
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
