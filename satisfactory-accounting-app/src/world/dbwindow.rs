use yew::{function_component, Html};

use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};

pub type DbChooserWindowManager = WindowManager<DbChooserWindow>;
pub type DbChooserWindowDispatcher = ShowWindowDispatcher<DbChooserWindow>;

#[function_component]
fn DbChooserWindow() -> Html {
    todo!()
}
