mod app;
mod node_display;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Unable to init logger");
    yew::start_app::<app::App>();
}
