use std::marker::PhantomData;
use std::rc::Rc;

use yew::{
    function_component, html, use_reducer_eq, BaseComponent, ContextProvider, Html, Properties,
    Reducible, UseReducerDispatcher,
};

/// Actions for a show window controller.
enum Action {
    /// Hide the window.
    Hide,
    /// Toggle the window.
    Toggle,
}

/// The state of a window.
#[derive(Default, PartialEq, Copy, Clone)]
struct ShowWindowState {
    /// Whether user settings are currently shown.
    show_window: bool,
}

impl Reducible for ShowWindowState {
    type Action = Action;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new_state = match action {
            Action::Hide => false,
            Action::Toggle => !self.show_window,
        };
        // Avoid allocating a new Rc if this is the only live instance.
        Rc::make_mut(&mut self).show_window = new_state;
        self
    }
}

/// Dispatcher for toggling the window.
pub struct ShowWindowDispatcher<T> {
    reducer: UseReducerDispatcher<ShowWindowState>,
    _type_tag: PhantomData<T>,
}

impl<T> ShowWindowDispatcher<T> {
    fn new(reducer: UseReducerDispatcher<ShowWindowState>) -> Self {
        Self {
            reducer,
            _type_tag: PhantomData,
        }
    }
}

impl<T> PartialEq for ShowWindowDispatcher<T> {
    fn eq(&self, other: &Self) -> bool {
        self.reducer == other.reducer
    }
}

impl<T> Clone for ShowWindowDispatcher<T> {
    fn clone(&self) -> Self {
        Self {
            reducer: self.reducer.clone(),
            _type_tag: self._type_tag,
        }
    }
}

impl<T> ShowWindowDispatcher<T> {
    /// Toggles the window.
    pub fn toggle_window(&self) {
        self.reducer.dispatch(Action::Toggle);
    }

    /// Hides the window.
    pub fn hide_window(&self) {
        self.reducer.dispatch(Action::Hide);
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Children to render outside of the window. These children have access to a context which can
    /// be used to toggle the window.
    pub children: Html,
}

/// Manager for a window of a particular type. Provides a context manager which can be used to
/// toggle or hide the window and renders its children, plus adds the window if the shown state is
/// true.
#[function_component]
pub fn WindowManager<T>(Props { children }: &Props) -> Html
where
    T: BaseComponent<Properties = ()>,
{
    let show_window = use_reducer_eq(ShowWindowState::default);
    let window_dispatcher = ShowWindowDispatcher::<T>::new(show_window.dispatcher());

    html! {
        <ContextProvider<ShowWindowDispatcher<T>> context={window_dispatcher}>
        { children.clone() }
        if show_window.show_window {
            <T />
        }
        </ContextProvider<ShowWindowDispatcher<T>>>
    }
}
