use wasm_bindgen::JsCast as _;
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::AttrValue;

/// Extract the text value from the target of an InputEvent.
pub fn get_value_from_input_event(e: InputEvent) -> AttrValue {
    let event: Event = e.dyn_into().unwrap();
    let event_target = event.target().unwrap();
    let target: HtmlInputElement = event_target.dyn_into().unwrap();
    target.value().into()
}
