use std::collections::HashSet;

use implicit_clone::ImplicitClone;
use log::{error, warn};
use yew::html::Scope;
use yew::{
    classes, function_component, hook, html, use_callback, use_context, AttrValue, Callback,
    Classes, Component, Context, ContextProvider, Html, Properties,
};

use crate::inputs::button::Button;
use crate::material::material_icon;
use crate::overlay_window::OverlayWindow;
use crate::refeqrc::RefEqRc;

/// Defines a modal dialog.
#[derive(Default, Debug)]
struct Modal {
    /// Title to display for the modal.
    title: AttrValue,
    /// Content to display in the modal.
    content: Html,
    /// Extra classes to add to the modal.
    class: Classes,
    /// Which kind of modal to display.
    kind: ModalKind,
}

/// Enum of supported modal kinds.
#[derive(Debug)]
pub enum ModalKind {
    /// A modal with a single "Ok" button.
    Ok(ModalOk),
    /// A modal with "Cancel" and "Delete" buttons.
    CancelDelete(CancelDelete),
    /// A modal with arbitrary binary choice buttons.
    BinaryChoice(BinaryChoice),
}

impl Default for ModalKind {
    fn default() -> Self {
        Self::Ok(Default::default())
    }
}

impl From<ModalOk> for ModalKind {
    fn from(value: ModalOk) -> Self {
        Self::Ok(value)
    }
}

impl From<CancelDelete> for ModalKind {
    fn from(value: CancelDelete) -> Self {
        Self::CancelDelete(value)
    }
}

impl From<BinaryChoice> for ModalKind {
    fn from(value: BinaryChoice) -> Self {
        Self::BinaryChoice(value)
    }
}

/// Modal settings specific to "Ok".
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModalOk {
    /// Callback to run on a click to "Ok", in addition to closing the modal.
    on_ok: Callback<()>,
}

impl ImplicitClone for ModalOk {}

impl ModalOk {
    /// Creates a ModalOk which just closes on "ok".
    pub fn close() -> Self {
        Default::default()
    }
}

/// Modal settings specific to CancelDelete.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CancelDelete {
    /// Callback to run on "cancel", in addition to closing the modal.
    on_cancel: Callback<()>,
    /// Callback to run on "delete", in addition to closing the modal.
    on_delete: Callback<()>,
}

impl ImplicitClone for CancelDelete {}

impl CancelDelete {
    /// Creates a [CancelDelete] that only has an on_delete callback.
    pub fn delete(on_delete: Callback<()>) -> Self {
        Self {
            on_delete,
            ..Default::default()
        }
    }
}

/// Modal buttons for an arbitrary binary choice.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryChoice {
    /// Label for the left hand choice on the modal.
    lhs: Html,
    /// Optional title to apply to the left hand choice.
    lhs_title: Option<AttrValue>,
    /// Callback for the left hand choice on the modal, in addition to closing the modal.
    on_lhs: Callback<()>,
    /// Label for the right hand choice on the modal.
    rhs: Html,
    /// Optional title to apply to the right hand choice.
    rhs_title: Option<AttrValue>,
    /// Callback for the right hand choice on the modal, in addition to closing the modal.
    on_rhs: Callback<()>,
}

impl ImplicitClone for BinaryChoice {}

impl BinaryChoice {
    /// Create a new binary choice.
    pub fn new(lhs: Html, rhs: Html) -> Self {
        BinaryChoice {
            lhs,
            lhs_title: None,
            on_lhs: Callback::noop(),
            rhs,
            rhs_title: None,
            on_rhs: Callback::noop(),
        }
    }

    /// Sets the title for the left hand option.
    pub fn lhs_title(mut self, lhs_title: impl Into<AttrValue>) -> Self {
        self.lhs_title = Some(lhs_title.into());
        self
    }

    /// Adds a callback for when the user clicks the left hand side choice. The callback runs in
    /// addition to closing the modal.
    pub fn on_lhs(mut self, on_lhs: Callback<()>) -> Self {
        self.on_lhs = on_lhs;
        self
    }

    /// Sets the title for the right hand option.
    pub fn rhs_title(mut self, rhs_title: impl Into<AttrValue>) -> Self {
        self.rhs_title = Some(rhs_title.into());
        self
    }

    /// Adds a callback for when the user clicks the right hand side choice. The callback runs in
    /// addition to closing the modal.
    pub fn on_rhs(mut self, on_rhs: Callback<()>) -> Self {
        self.on_rhs = on_rhs;
        self
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Children which should have access to the ModalDispatcher.
    pub children: Html,
}

pub enum Msg {
    /// Create a modal a populate its handle's ID.
    CreateModal {
        /// Content to display in the modal.
        #[allow(private_interfaces)] // Use ModalDispatcher instead.
        modal: RefEqRc<Modal>,
    },
    /// Delete a modal.
    DeleteModal {
        /// Reference to the modal to remove.
        #[allow(private_interfaces)] // Use ModalDispatcher instead.
        modal: RefEqRc<Modal>,
    },
}

/// Manages modal dialogs.
pub struct ModalManager {
    /// Currently displayed modals.
    unordered_modals: HashSet<RefEqRc<Modal>>,
    /// Currently displayed modals in insertion order.
    ordered_modals: Vec<RefEqRc<Modal>>,

    /// Scope used to send messages to Self.
    link: RefEqRc<Scope<Self>>,
}

impl ModalManager {
    /// Message handler for CreateModal.
    fn create_modal(&mut self, modal: RefEqRc<Modal>) -> bool {
        if self.unordered_modals.insert(modal.clone()) {
            self.ordered_modals.push(modal);
            true
        } else {
            warn!("Already contained this modal: {modal:?}");
            false
        }
    }

    /// Message handler for DeleteModal.
    fn delete_modal(&mut self, modal: RefEqRc<Modal>) -> bool {
        if self.unordered_modals.remove(&modal) {
            if let Some(idx) = self.ordered_modals.iter().position(|m| *m == modal) {
                self.ordered_modals.remove(idx);
            } else {
                error!(
                    "This modal was in the unordered_modals but not in the ordered modals: \
                    {modal:?}"
                );
            }
            true
        } else {
            // This is the expected state if the modal is closed internally before the handle is
            // dropped.
            false
        }
    }

    /// Gets the [`ModalDispatcher`] for this [`ModalManager`].
    fn dispatcher(&self) -> ModalDispatcher {
        ModalDispatcher {
            link: self.link.clone(),
        }
    }
}

impl Component for ModalManager {
    type Properties = Props;
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            unordered_modals: HashSet::with_capacity(10),
            ordered_modals: Vec::with_capacity(10),
            link: RefEqRc::new(ctx.link().clone()),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CreateModal { modal } => self.create_modal(modal),
            Msg::DeleteModal { modal } => self.delete_modal(modal),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let modals = self.ordered_modals.iter().map(|modal| {
            let id = RefEqRc::as_ptr(modal) as usize;
            html! {
                // Tag with the pointer identity to prevent modals from being recreated if earlier
                // modals are dismissed.
                <ModalWindow key={id} modal={modal.clone()} />
            }
        });

        html! {
            <>
                <ContextProvider<ModalDispatcher> context={self.dispatcher()}>
                // Place the modals in a fragment so that the differ doesn't think it needs to
                // re-create/re-render the rest of the children.
                <>
                    {for modals}
                </>
                {ctx.props().children.clone()}
                </ContextProvider<ModalDispatcher>>
            </>
        }
    }
}

#[derive(Properties, PartialEq)]
struct ModalWindowProps {
    /// The actual modal to display.
    modal: RefEqRc<Modal>,
}

/// Actual modal window.
#[function_component]
fn ModalWindow(ModalWindowProps { modal }: &ModalWindowProps) -> Html {
    let dispatcher = use_modal_dispatcher();
    let close_window = use_callback((modal.clone(), dispatcher), |(), (modal, dispatcher)| {
        dispatcher.close_modal(modal.clone());
    });

    let class = classes!("ModalWindow", modal.class.clone());
    let buttons = match &modal.kind {
        ModalKind::Ok(modal_ok) => html! {
            <ModalOkDisplay {modal_ok} {close_window} />
        },
        ModalKind::CancelDelete(cancel_delete) => html! {
            <CancelDeleteDisplay {cancel_delete} {close_window} />
        },
        ModalKind::BinaryChoice(binary_choice) => html! {
            <BinaryChoiceDisplay {binary_choice} {close_window} />
        },
    };
    html! {
        <OverlayWindow title={&modal.title} {class}>
            {modal.content.clone()}
            <div class="modal-buttons">
                {buttons}
            </div>
        </OverlayWindow>
    }
}

#[derive(PartialEq, Properties)]
pub struct ModalOkProps {
    close_window: Callback<()>,
    modal_ok: ModalOk,
}

/// Display for the ModalOk.
#[function_component]
fn ModalOkDisplay(props: &ModalOkProps) -> Html {
    let onclick = use_callback(
        (props.close_window.clone(), props.modal_ok.on_ok.clone()),
        |(), (close_window, on_ok)| {
            close_window.emit(());
            on_ok.emit(());
        },
    );
    html! {
        <Button class="modal-button" title="Ok" {onclick}>
            {material_icon("check")}
            {"Ok"}
        </Button>
    }
}

#[derive(PartialEq, Properties)]
pub struct CancelDeleteProps {
    close_window: Callback<()>,
    cancel_delete: CancelDelete,
}

/// Display for CancelDelete.
#[function_component]
fn CancelDeleteDisplay(props: &CancelDeleteProps) -> Html {
    let cancel = use_callback(
        (
            props.close_window.clone(),
            props.cancel_delete.on_cancel.clone(),
        ),
        |(), (close_window, on_cancel)| {
            close_window.emit(());
            on_cancel.emit(());
        },
    );
    let delete = use_callback(
        (
            props.close_window.clone(),
            props.cancel_delete.on_delete.clone(),
        ),
        |(), (close_window, on_delete)| {
            close_window.emit(());
            on_delete.emit(());
        },
    );
    html! {
        <>
            <Button class="green modal-button" title="Ok" onclick={cancel}>
                {material_icon("check")}
                <span>{"Cancel"}</span>
            </Button>
            <Button class="red modal-button" title="Ok" onclick={delete}>
                {material_icon("delete_forever")}
                <span>{"Delete"}</span>
            </Button>
        </>
    }
}

#[derive(PartialEq, Properties)]
pub struct BinaryChoiceProps {
    close_window: Callback<()>,
    binary_choice: BinaryChoice,
}

/// Display for CancelDelete.
#[function_component]
fn BinaryChoiceDisplay(props: &BinaryChoiceProps) -> Html {
    let on_lhs = use_callback(
        (
            props.close_window.clone(),
            props.binary_choice.on_lhs.clone(),
        ),
        |(), (close_window, on_lhs)| {
            close_window.emit(());
            on_lhs.emit(());
        },
    );
    let on_rhs = use_callback(
        (
            props.close_window.clone(),
            props.binary_choice.on_rhs.clone(),
        ),
        |(), (close_window, on_rhs)| {
            close_window.emit(());
            on_rhs.emit(());
        },
    );
    html! {
        <>
            <Button class="modal-button" title={&props.binary_choice.lhs_title} onclick={on_lhs}>
                {props.binary_choice.lhs.clone()}
            </Button>
            <Button class="modal-button" title={&props.binary_choice.rhs_title} onclick={on_rhs}>
                {props.binary_choice.rhs.clone()}
            </Button>
        </>
    }
}

/// Dispatcher used to control modal dialogs.
#[derive(Debug, Clone, PartialEq)]
pub struct ModalDispatcher {
    /// Link used to send messages to the ModalManager.
    link: RefEqRc<Scope<ModalManager>>,
}

impl ModalDispatcher {
    /// Creates a modal dialog and returns a handle to it which must be kept around as long as you
    /// want the modal to remain shown.
    pub fn builder(&self) -> ModalBuilder {
        ModalBuilder {
            dispatcher: self,
            modal: Default::default(),
        }
    }

    /// Creates a new modal.
    fn create_modal(&self, modal: Modal) -> ModalHandle {
        let modal = RefEqRc::new(modal);
        self.link.send_message(Msg::CreateModal {
            modal: modal.clone(),
        });
        ModalHandle {
            modal: Some(modal),
            dispatcher: self.clone(),
        }
    }

    /// Closes a modal by id.
    fn close_modal(&self, modal: RefEqRc<Modal>) {
        self.link.send_message(Msg::DeleteModal { modal });
    }
}

/// Builder for modal dialogs.
pub struct ModalBuilder<'d> {
    dispatcher: &'d ModalDispatcher,
    modal: Modal,
}

impl<'d> ModalBuilder<'d> {
    /// Sets the title to display on the Modal.
    pub fn title<A: Into<AttrValue>>(mut self, title: A) -> Self {
        self.modal.title = title.into();
        self
    }

    /// Set the content of the modal.
    pub fn content(mut self, content: Html) -> Self {
        self.modal.content = content;
        self
    }

    /// Set the extra classes to display on the modal.
    pub fn class<C: Into<Classes>>(mut self, class: C) -> Self {
        self.modal.class = class.into();
        self
    }

    /// Sets the ModalKind.
    pub fn kind<K: Into<ModalKind>>(mut self, kind: K) -> Self {
        self.modal.kind = kind.into();
        self
    }

    /// Builds the modal and displays it. Returns a handle which will keep the modal or clean it up
    /// when dropped.
    pub fn build(self) -> ModalHandle {
        self.dispatcher.create_modal(self.modal)
    }
}

/// Gets the dispatcher for creating modal dialogs.
#[hook]
pub fn use_modal_dispatcher() -> ModalDispatcher {
    use_context::<ModalDispatcher>()
        .expect("use_modal_dispatcher can only be used from a child of ModalManager")
}

/// Handle to a Modal dialog. The modal will be removed when this handle is dropped.
pub struct ModalHandle {
    /// Shared ID of this modal. This is not initially populated but will be populated when the
    /// modal is created.
    modal: Option<RefEqRc<Modal>>,

    /// Link to the modal manager, used to close the modal when dropped.
    dispatcher: ModalDispatcher,
}

impl ModalHandle {
    /// Consumes the modal handle and allows the modal to persist rather than disappearing when the
    /// handle is dropped.
    pub fn persist(mut self) {
        self.modal = None;
    }
}

impl Drop for ModalHandle {
    fn drop(&mut self) {
        if let Some(modal) = self.modal.take() {
            self.dispatcher.close_modal(modal);
        }
    }
}
